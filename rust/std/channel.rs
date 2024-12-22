use core::marker::PhantomData;
use core::mem::size_of;
use core::ops::Drop;
use core::ptr;
use prelude::*;
use sys::{
	alloc, channel_destroy, channel_handle_size, channel_init, channel_recv, channel_send, release,
	Message,
};

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
			channel_destroy(self.handle);
			release(self.handle);
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
			let mut b = Box::new(t).unwrap();
			(*msg).payload = b.as_mut_ptr() as *mut u8;
			b.leak();
			channel_send(self.inner.handle, msg as *mut u8);
			Ok(())
		}
	}

	pub fn recv(&self) -> Result<T, Error> {
		unsafe {
			let recv = channel_recv(self.inner.handle) as *mut Message;
			let payload = (*recv).payload as *mut T;
			let mut nbox = Box::from_raw(payload);
			nbox.leak();
			let v = ptr::read(nbox.into_inner());
			release(payload as *mut u8);
			release(recv as *mut u8);
			Ok(v)
		}
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
			spawn(|| {
				let v = channel.recv().unwrap();
				assert_eq!(v, 101);
				let _ = lock.read(); // memory fence only
				assert_eq!(*rc_clone, 1);
				*rc_clone += 1;
				assert_eq!(*rc_clone, 2);
			});

			channel.send(101);

			loop {
				let _ = lock.read(); // memory fence only
				if *rc == 1 {
				} else {
					assert_eq!(*rc, 2);
					break;
				}
			}
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
			let channel2 = channel.clone().unwrap();
			let lock = lock!();
			let lock2 = lock!();
			let rc = Rc::new(1).unwrap();
			let mut rc_clone = rc.clone().unwrap();
			spawn(move || {
				let v = channel2.recv().unwrap();
				assert_eq!(v, 101);
				let _ = lock2.read(); // memory fence only
				assert_eq!(*rc_clone, 1);
				*rc_clone += 1;
				assert_eq!(*rc_clone, 2);
			});

			channel.send(101);

			loop {
				let _ = lock.read(); // memory fence only
				if *rc == 1 {
				} else {
					assert_eq!(*rc, 2);
					break;
				}
			}
		}
		assert_eq!(initial, unsafe { getalloccount() });
	}
}
