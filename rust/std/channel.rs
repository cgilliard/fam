use core::marker::PhantomData;
use core::mem::size_of;
use core::ops::Drop;
use core::ptr;
use prelude::*;
use sys::{
	alloc, channel_destroy, channel_handle_size, channel_init, channel_pending, channel_recv,
	channel_send, release, Message,
};

#[macro_export]
macro_rules! channel {
	() => {{
		let channel = Channel::new();
		match channel {
			Ok(sender) => match sender.clone() {
				Ok(receiver) => Ok((sender, receiver)),
				Err(e) => Err(e),
			},
			Err(e) => Err(e),
		}
	}};
}

struct ChannelInner<T> {
	handle: *mut u8,
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
		unsafe {
			while channel_pending(self.handle) {
				let _recv = self.recv();
			}
			channel_destroy(self.handle);
			release(self.handle);
		}
	}
}

impl<T> ChannelInner<T> {
	pub fn recv(&self) -> Result<T, Error> {
		unsafe {
			let recv = channel_recv(self.handle) as *mut Message;
			let payload = (*recv).payload as *mut T;
			let mut nbox = Box::from_raw(Pointer::new(payload));
			nbox.leak();
			let v = ptr::read(nbox.as_ptr().raw());
			if !payload.is_null() {
				release(payload as *mut u8);
			}
			if !recv.is_null() {
				release(recv as *mut u8);
			}
			Ok(v)
		}
	}
}

impl<T> Channel<T> {
	pub fn new() -> Result<Channel<T>, Error> {
		if unsafe { channel_handle_size() } > 128 {
			exit!("channel_handle_size() > 128");
		}
		unsafe {
			let handle = alloc(channel_handle_size());
			let ret = match Rc::new(ChannelInner {
				handle,
				_marker: PhantomData,
			}) {
				Ok(inner) => Self { inner },
				Err(e) => return Err(e),
			};

			channel_init(ret.inner.handle);

			Ok(ret)
		}
	}

	pub fn send(&self, t: T) -> Result<(), Error> {
		unsafe {
			let msg = alloc(size_of::<Message>()) as *mut Message;
			match Box::new(t) {
				Ok(mut b) => {
					(*msg).payload = b.as_ptr().raw() as *mut u8;
					b.leak();
					channel_send(self.inner.handle, msg as *mut u8);
					Ok(())
				}
				Err(e) => Err(e),
			}
		}
	}

	pub fn recv(&self) -> Result<T, Error> {
		self.inner.recv()
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use sys::getalloccount;

	#[test]
	fn test_channel_std() {
		let initial = unsafe { getalloccount() };
		{
			let channel = Channel::new().unwrap();
			let lock = lock!();
			let rc = Rc::new(1).unwrap();
			let mut rc_clone = rc.clone().unwrap();
			let mut jh = spawnj(|| {
				let v = channel.recv().unwrap();
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
		assert_eq!(initial, unsafe { getalloccount() });
	}

	#[test]
	fn test_channel_clone() {
		let initial = unsafe { getalloccount() };
		{
			let channel: Channel<u32> = Channel::new().unwrap();
			let _channel2 = channel.clone().unwrap();
		}
		assert_eq!(initial, unsafe { getalloccount() });
	}

	#[test]
	fn test_channel_move_std() {
		let initial = unsafe { getalloccount() };
		{
			let channel = Channel::new().unwrap();
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
		assert_eq!(initial, unsafe { getalloccount() });
	}

	#[test]
	fn test_channel_result() {
		let initial = unsafe { getalloccount() };
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
		assert_eq!(initial, unsafe { getalloccount() });
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
		let initial = unsafe { getalloccount() };
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
		assert_eq!(initial, unsafe { getalloccount() });
		assert_eq!(unsafe { DROPCOUNT }, 2);
		assert_eq!(unsafe { DROPSUM }, 305);
	}

	#[test]
	fn test_cleanup() {
		let initial = unsafe { getalloccount() };
		{
			let channel = Channel::new().unwrap();
			channel.send(0).unwrap();
			channel.send(0).unwrap();
		}
		assert_eq!(initial, unsafe { getalloccount() });
	}
}
