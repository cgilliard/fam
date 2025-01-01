use core::marker::PhantomData;
use core::mem::{forget, size_of, zeroed, ManuallyDrop};
use core::ops::Drop;
use core::ptr;
use core::ptr::null_mut;
use prelude::*;
use sys::{
	safe_alloc, safe_channel_destroy, safe_channel_handle_size, safe_channel_init,
	safe_channel_recv, safe_channel_send, safe_release, MessageHeader,
};

struct Message<T> {
	_header: MessageHeader,
	value: T,
}

struct ChannelInner<T> {
	handle: Ptr<u8>,
	_marker: PhantomData<T>,
}

pub struct Channel<T> {
	inner: Rc<ChannelInner<T>>,
}

impl<T> Clone for Channel<T> {
	fn clone(&self) -> Result<Self, Error> {
		match self.inner.clone() {
			Ok(inner) => Ok(Channel { inner }),
			Err(e) => Err(e),
		}
	}
}

impl<T> Drop for ChannelInner<T> {
	fn drop(&mut self) {
		if !self.handle.raw().is_null() {
			safe_channel_destroy(self.handle.raw());
			safe_release(self.handle.raw());
		}
	}
}

impl<T> ChannelInner<T> {
	pub fn recv(&self) -> Result<T, Error> {
		let mut msg_storage: Message<T> = unsafe { zeroed() };
		let msg_ptr = &mut msg_storage as *mut Message<T>;
		safe_channel_recv(self.handle.raw(), msg_ptr as *mut u8);
		let result = unsafe { ptr::read(&msg_storage.value) };
		forget(msg_storage);
		Ok(result)
	}

	pub fn send(&self, value: T) -> Result<(), Error> {
		let msg = ManuallyDrop::new(Message {
			_header: MessageHeader {
				_reserved: null_mut(),
			},
			value,
		});

		let msg_ptr = &msg as *const ManuallyDrop<Message<T>> as *const u8;
		if safe_channel_send(self.handle.raw(), msg_ptr) < 0 {
			Err(err!(CapacityExceeded))
		} else {
			Ok(())
		}
	}
}

impl<T> Channel<T> {
	pub fn new(capacity: u64) -> Result<Channel<T>, Error> {
		let size = size_of::<T>();
		let handle = safe_alloc(safe_channel_handle_size() + size * (1 + capacity) as usize);
		let handle = if handle.is_null() {
			return Err(err!(Alloc));
		} else {
			Ptr::new(handle)
		};

		let ret = match Rc::new(ChannelInner {
			handle,
			_marker: PhantomData,
		}) {
			Ok(inner) => Self { inner },
			Err(e) => return Err(e),
		};

		let res = safe_channel_init(ret.inner.handle.raw(), size as u64, 1 + capacity);
		if res != 0 {
			return Err(err!(ChannelInit));
		}

		Ok(ret)
	}

	pub fn send(&self, t: T) -> Result<(), Error> {
		self.inner.send(t)
	}

	pub fn recv(&self) -> Result<T, Error> {
		self.inner.recv()
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use sys::safe_getalloccount;

	#[test]
	fn test_channel_std() {
		let initial = safe_getalloccount();
		{
			let channel = Channel::new(1024).unwrap();
			let lock = lock!();
			let rc = Rc::new(1).unwrap();
			let mut rc_clone = rc.clone().unwrap();
			let mut jh = spawnj(|| {
				let v = channel.recv().unwrap();
				assert_eq!(v, 101);
				let _v = lock.write();
				assert_eq!(*rc_clone, 1);
				*rc_clone += 1;
				assert_eq!(*rc_clone, 2);
			})
			.unwrap();
			channel.send(101).unwrap();

			loop {
				{
					let _v = lock.read();
					if *rc == 1 {
					} else {
						assert_eq!(*rc, 2);
						break;
					}
				}
				unsafe {
					crate::sys::sleep_millis(1);
				}
			}
			assert!(jh.join().is_ok());
		}
		assert_eq!(initial, safe_getalloccount());
	}

	#[test]
	fn test_channel_clone() {
		let initial = safe_getalloccount();
		{
			let channel: Channel<u32> = Channel::new(1024).unwrap();
			let _channel2 = channel.clone().unwrap();
		}
		assert_eq!(initial, safe_getalloccount());
	}

	#[test]
	fn test_channel_move_std() {
		let initial = safe_getalloccount();
		{
			let channel = Channel::new(1024).unwrap();
			let channel_clone = channel.clone().unwrap();
			let lock = lock_box!().unwrap();
			let lock_clone = lock.clone().unwrap();
			let rc = Rc::new(1).unwrap();
			let mut rc_clone = rc.clone().unwrap();
			let mut jh = spawnj(move || {
				let v = { channel_clone.recv().unwrap() };
				assert_eq!(v, 101);
				let _v = lock_clone.write();
				assert_eq!(*rc_clone, 1);
				*rc_clone += 1;
				assert_eq!(*rc_clone, 2);
			})
			.unwrap();

			channel.send(101).unwrap();

			loop {
				{
					let _v = lock.read();
					if *rc == 1 {
					} else {
						assert_eq!(*rc, 2);
						break;
					}
				}
				unsafe {
					crate::sys::sleep_millis(1);
				}
			}
			assert!(jh.join().is_ok());
		}
		assert_eq!(initial, safe_getalloccount());
	}

	#[test]
	fn test_channel_result() {
		let initial = safe_getalloccount();
		{
			let channel = Channel::new(1024).unwrap();
			let channel_clone = channel.clone().unwrap();
			let channel2 = Channel::new(1024).unwrap();
			let channel2_clone = channel2.clone().unwrap();
			let lock = lock_box!().unwrap();
			let lock_clone = lock.clone().unwrap();
			let rc = Rc::new(0).unwrap();
			let mut rc_clone = rc.clone().unwrap();

			let mut jh = spawnj(move || {
				{
					let input = channel_clone.recv().unwrap();
					let _v = lock_clone.write();
					*rc_clone = input + 100;
				}
				channel2_clone.send(()).unwrap();
			})
			.unwrap();

			channel.send(301).unwrap();
			let result = channel2.recv().unwrap();

			assert_eq!(result, ());
			assert_eq!(*rc, 401);

			assert!(jh.join().is_ok());
		}
		assert_eq!(initial, safe_getalloccount());
	}

	struct DropTest {
		x: u32,
	}

	static mut DROPCOUNT: u32 = 0;
	static mut DROPSUM: u32 = 0;

	impl Drop for DropTest {
		fn drop(&mut self) {
			unsafe {
				DROPCOUNT += 1;
				DROPSUM += self.x;
			}
		}
	}

	#[test]
	fn test_send_zero_sized() {
		let initial = safe_getalloccount();
		{
			let channel1a = Channel::new(1024).unwrap();
			let channel1b = channel1a.clone().unwrap();
			let channel2a = Channel::new(1024).unwrap();
			let channel2b = channel2a.clone().unwrap();

			let mut jh = spawnj(move || {
				assert_eq!(channel1b.recv().unwrap(), ());
				channel2b.send(()).unwrap();
			})
			.unwrap();

			channel1a.send(()).unwrap();

			assert_eq!(channel2a.recv().unwrap(), ());

			assert!(jh.join().is_ok());
		}
		assert_eq!(safe_getalloccount(), initial);
	}

	#[test]
	fn test_channel_drop() {
		let initial = safe_getalloccount();
		{
			let channel = Channel::new(1024).unwrap();
			let channel_clone = channel.clone().unwrap();
			let channel2 = Channel::new(1024).unwrap();
			let channel2_clone = channel2.clone().unwrap();
			let lock = lock_box!().unwrap();
			let lock_clone = lock.clone().unwrap();
			let rc = Rc::new(0).unwrap();
			let mut rc_clone = rc.clone().unwrap();

			let mut jh = spawnj(move || {
				{
					let input: DropTest = channel_clone.recv().unwrap();
					let _v = lock_clone.write();
					*rc_clone = input.x + 100;
					assert_eq!(unsafe { DROPCOUNT }, 0);
				}

				assert_eq!(unsafe { DROPCOUNT }, 1);
				channel2_clone.send(DropTest { x: 4 }).unwrap();
			})
			.unwrap();

			channel.send(DropTest { x: 301 }).unwrap();

			let result = channel2.recv().unwrap();

			assert_eq!(result.x, 4);
			assert_eq!(*rc, 401);
			assert!(jh.join().is_ok());
			assert_eq!(unsafe { DROPCOUNT }, 1);
		}
		assert_eq!(initial, safe_getalloccount());
		assert_eq!(unsafe { DROPCOUNT }, 2);
		assert_eq!(unsafe { DROPSUM }, 305);
	}

	#[test]
	fn test_cleanup() {
		let initial = safe_getalloccount();
		{
			let channel = Channel::new(1024).unwrap();
			channel.send(0).unwrap();
			channel.send(0).unwrap();
		}
		assert_eq!(initial, safe_getalloccount());
	}

	#[test]
	fn test_channel_cap_exceed() {
		let initial = safe_getalloccount();
		{
			let channel = Channel::new(1).unwrap();
			channel.send(0).unwrap();
			assert!(channel.send(0).is_err());
		}
		assert_eq!(initial, safe_getalloccount());
	}
}
