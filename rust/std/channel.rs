use core::marker::PhantomData;
use core::ops::Drop;
use core::ptr;
use core::ptr::null_mut;
use prelude::*;
use sys::{
	safe_alloc, safe_channel_destroy, safe_channel_handle_size, safe_channel_init,
	safe_channel_pending, safe_channel_recv, safe_channel_send, safe_release, MessageHeader,
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
		while safe_channel_pending(self.handle.raw()) {
			let _recv = self.recv();
		}
		safe_channel_destroy(self.handle.raw());
		safe_release(self.handle.raw());
	}
}

impl<T> ChannelInner<T> {
	pub fn recv(&self) -> Result<T, Error> {
		// recv the Box<Message<T>> and cast to Message<T>.
		// since Box uses the same memory layout as stack, we can do
		// this cast and just release the memory after reading the
		// value back onto the stack
		let recv = safe_channel_recv(self.handle.raw(), null_mut()) as *mut Message<T>;
		let msg = unsafe { ptr::read(recv) };
		safe_release(recv as *mut u8);
		Ok(msg.value)
	}

	pub fn send(&self, t: T) -> Result<(), Error> {
		let mut msg = match Box::new(Message {
			_header: MessageHeader { _next: null_mut() },
			value: t,
		}) {
			Ok(msg) => msg,
			Err(e) => return Err(e),
		};

		// Leak the box so that the other thread can get it and
		// accept ownership. This also prevents the drop handlers
		// from being called.
		msg.leak();
		safe_channel_send(self.handle.raw(), msg.as_ptr().raw() as *mut u8);
		Ok(())
	}
}

impl<T> Channel<T> {
	pub fn new() -> Result<Channel<T>, Error> {
		let handle = safe_alloc(safe_channel_handle_size());
		let handle = if handle.is_null() {
			return Err(ErrorKind::Alloc.into());
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

		let res = safe_channel_init(ret.inner.handle.raw());
		if res != 0 {
			return Err(ErrorKind::ChannelInit.into());
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
	fn test_send_zero_sized() {
		let initial = unsafe { getalloccount() };
		{
			let channel1a = Channel::new().unwrap();
			let channel1b = channel1a.clone().unwrap();
			let channel2a = Channel::new().unwrap();
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
		assert_eq!(unsafe { getalloccount() }, initial);
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
