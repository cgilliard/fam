use core::marker::PhantomData;
use core::ops::Drop;
use core::ptr;
use prelude::*;
use sys::{
	safe_channel_destroy, safe_channel_handle_size, safe_channel_init, safe_channel_pending,
	safe_channel_recv, safe_channel_send, safe_release,
};

#[repr(C)]
struct ChannelMessage<T> {
	_reserved: u128,
	_reserved2: u128,
	value: T,
}

struct ChannelInner<T> {
	handle: [u8; 128],
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
		while safe_channel_pending(&self.handle as *const u8) {
			let _recv = self.recv();
		}
		let handle = &self.handle;
		safe_channel_destroy(handle as *const u8);
	}
}

impl<T> ChannelInner<T> {
	pub fn recv(&self) -> T {
		let handle = &self.handle;
		let recv = safe_channel_recv(handle as *const u8) as *mut ChannelMessage<T>;
		let ptr = Ptr::new(recv);
		let mut nbox = Box::from_raw(ptr);
		nbox.leak();
		let v = unsafe { ptr::read(nbox.as_ptr().raw()) };
		safe_release(recv as *mut u8);
		v.value
	}

	pub fn send(&self, value: T) -> Result<(), Error> {
		let msg = ChannelMessage {
			_reserved: 0,
			_reserved2: 0,
			value,
		};
		match Box::new(msg) {
			Ok(mut b) => {
				b.leak();
				let handle = &self.handle;
				if safe_channel_send(handle as *const u8, b.as_ptr().raw() as *mut u8) < 0 {
					Err(err!(ChannelSend))
				} else {
					Ok(())
				}
			}
			Err(e) => Err(e),
		}
	}
}

impl<T> Channel<T> {
	pub fn new() -> Result<Channel<T>, Error> {
		if safe_channel_handle_size() > 128 {
			exit!("safe_channel_handle_size() > 128");
		}
		let handle = [0u8; 128];
		let mut ret = match Rc::new(ChannelInner {
			handle,
			_marker: PhantomData,
		}) {
			Ok(inner) => Self { inner },
			Err(e) => return Err(e),
		};

		let handle = &mut ret.inner.handle;
		if safe_channel_init(handle as *mut u8) < 0 {
			Err(err!(ChannelInit))
		} else {
			Ok(ret)
		}
	}

	pub fn send(&self, t: T) -> Result<(), Error> {
		self.inner.send(t)
	}

	pub fn recv(&self) -> T {
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
			let channel = Channel::new().unwrap();
			let lock = lock!();
			let rc = Rc::new(1).unwrap();
			let mut rc_clone = rc.clone().unwrap();
			let mut jh = spawnj(|| {
				let v = channel.recv();
				assert_eq!(v, 101);
				let _v = lock.write(); // memory fence only
				assert_eq!(*rc_clone, 1);
				*rc_clone += 1;
				assert_eq!(*rc_clone, 2);
			})
			.unwrap();

			channel.send(101).unwrap();

			loop {
				{
					let _v = lock.read(); // memory fence only
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
			let channel: Channel<u32> = Channel::new().unwrap();
			let _channel2 = channel.clone().unwrap();
		}
		assert_eq!(initial, safe_getalloccount());
	}

	#[test]
	fn test_channel_move_std() {
		let initial = safe_getalloccount();
		{
			let channel = Channel::new().unwrap();
			let channel_clone = channel.clone().unwrap();
			let lock = lock_box!().unwrap();
			let lock_clone = lock.clone().unwrap();
			let rc = Rc::new(1).unwrap();
			let mut rc_clone = rc.clone().unwrap();
			let mut jh = spawnj(move || {
				let v = channel_clone.recv();
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
			let channel = Channel::new().unwrap();
			let channel_clone = channel.clone().unwrap();
			let channel2 = Channel::new().unwrap();
			let channel2_clone = channel2.clone().unwrap();
			let lock = lock_box!().unwrap();
			let lock_clone = lock.clone().unwrap();
			let rc = Rc::new(0).unwrap();
			let mut rc_clone = rc.clone().unwrap();

			let mut jh = spawnj(move || {
				{
					let input = channel_clone.recv();
					let _v = lock_clone.write();
					*rc_clone = input + 100;
				}
				channel2_clone.send(()).unwrap();
			})
			.unwrap();

			channel.send(301).unwrap();
			let result = channel2.recv();

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
	fn test_channel_drop() {
		let initial = safe_getalloccount();
		{
			let channel = Channel::new().unwrap();
			let channel_clone = channel.clone().unwrap();
			let channel2 = Channel::new().unwrap();
			let channel2_clone = channel2.clone().unwrap();
			let lock = lock_box!().unwrap();
			let lock_clone = lock.clone().unwrap();
			let rc = Rc::new(0).unwrap();
			let mut rc_clone = rc.clone().unwrap();

			let mut jh = spawnj(move || {
				{
					let input: DropTest = channel_clone.recv();
					let _v = lock_clone.write();
					*rc_clone = input.x + 100;
					assert_eq!(unsafe { DROPCOUNT }, 0);
				}
				assert_eq!(unsafe { DROPCOUNT }, 1);
				channel2_clone.send(DropTest { x: 4 }).unwrap();
			})
			.unwrap();

			channel.send(DropTest { x: 301 }).unwrap();
			let result = channel2.recv();

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
			let channel = Channel::new().unwrap();
			channel.send(0).unwrap();
			channel.send(0).unwrap();
		}
		assert_eq!(initial, safe_getalloccount());
	}

	#[test]
	fn test_multisend_chan() {
		let initial = safe_getalloccount();
		{
			let channel = Channel::new().unwrap();
			let recv = channel.clone().unwrap();
			channel.send(0).unwrap();
			channel.send(1).unwrap();
			channel.send(2).unwrap();
			channel.send(3).unwrap();
			channel.send(4).unwrap();
			channel.send(5).unwrap();

			assert_eq!(recv.recv(), 0);
			assert_eq!(recv.recv(), 1);
			assert_eq!(recv.recv(), 2);
		}
		assert_eq!(initial, safe_getalloccount());
	}
}
