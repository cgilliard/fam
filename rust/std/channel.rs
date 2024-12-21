use core::marker::PhantomData;
use core::ops::Drop;
use core::ptr;
use prelude::*;
use sys::{channel_init, channel_recv, channel_send, map, unmap, Message};

pub struct Channel<T> {
	handle: *mut u8,
	_marker: PhantomData<T>,
}

impl<T> Drop for Channel<T> {
	fn drop(&mut self) {
		unsafe {
			unmap(self.handle, 1);
		}
	}
}

impl<T> Channel<T> {
	pub fn new() -> Result<Channel<T>, Error> {
		unsafe {
			let handle = map(1);
			channel_init(handle);
			Ok(Channel {
				handle,
				_marker: PhantomData,
			})
		}
	}

	pub fn send(&self, t: T) -> Result<(), Error> {
		unsafe {
			let msg = map(1) as *mut Message;
			let payload = map(1) as *mut u64;
			let mut b = Box::new(t).unwrap();
			(*payload.add(0)) = b.as_mut_ptr() as u64;
			(*payload.add(1)) = b.metadata();
			b.leak();
			(*msg).payload = payload as *mut u8;
			channel_send(self.handle, msg as *mut u8);
			Ok(())
		}
	}

	pub fn recv(&self) -> Result<T, Error> {
		unsafe {
			let recv = channel_recv(self.handle) as *mut Message;
			let payload = (*recv).payload as *mut u64;
			let ptr = *payload.add(0) as *mut T;
			let metadata = *payload.add(1) as u64;
			let nbox = Box::from_raw(ptr, metadata);
			unmap(payload as *mut u8, 1);
			unmap(recv as *mut u8, 1);
			Ok(ptr::read(nbox.into_inner()))
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use std::boxed::assert_all_slabs_free;
	use sys::getalloccount;

	#[test]
	fn test_channel_std() {
		let initial = unsafe { getalloccount() };
		{
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
			assert_all_slabs_free();
			unsafe {
				cleanup_slab_allocators();
			}
		}
		assert_eq!(initial, unsafe { getalloccount() });
	}
}
