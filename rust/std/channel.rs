use core::marker::PhantomData;
use core::mem::size_of;
use core::ops::Drop;
use core::ptr;
use prelude::*;
use std::boxed::get_slab_allocator;
use std::slabs::Slab;
use sys::{channel_init, channel_recv, channel_send, map, unmap, Message};

pub struct Channel<T> {
	handle: *mut u8,
	_marker: PhantomData<T>,
}

struct Payload<T> {
	bptr: *mut T,
	metadata: u64,
	selfptr: *mut u8,
	id: usize,
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
		let mut b = match Box::new(t) {
			Ok(b) => b,
			Err(e) => return Err(e),
		};
		let sa = match get_slab_allocator(size_of::<Payload<T>>()) {
			Some(sa) => sa,
			None => return Err(ErrorKind::Alloc.into()),
		};
		unsafe {
			let msg = map(1) as *mut Message;

			let slab = match sa.alloc() {
				Ok(slab) => slab,
				Err(e) => return Err(e),
			};

			ptr::write(
				slab.get_raw() as *mut Payload<T>,
				Payload {
					bptr: b.as_mut_ptr() as *mut T,
					metadata: b.metadata(),
					selfptr: slab.get_raw(),
					id: slab.get_id(),
				},
			);

			(*msg).payload = slab.get_raw() as *mut u8;
			channel_send(self.handle, msg as *mut u8);

			b.leak();
			Ok(())
		}
	}

	pub fn recv(&self) -> Result<T, Error> {
		let sa = match get_slab_allocator(size_of::<Payload<T>>()) {
			Some(sa) => sa,
			None => exit!("expected slab allocator not found!"),
		};
		unsafe {
			let recv = channel_recv(self.handle) as *mut Message;
			if recv.is_null() {
				return Err(ErrorKind::ChannelRecv.into());
			}
			let payload = (*recv).payload as *mut Payload<T>;
			let ptr = (*payload).bptr;
			let metadata = (*payload).metadata;
			let nbox = Box::from_raw(ptr, metadata);
			let mut slab = Slab::from_raw((*payload).selfptr, (*payload).id);
			sa.free(&mut slab);

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
				let channel2 = Channel::new().unwrap();
				let rc = Rc::new(1).unwrap();
				let mut rc_clone = rc.clone().unwrap();
				spawn(|| {
					let v = channel.recv().unwrap();
					assert_eq!(v, 101);
					assert_eq!(*rc_clone, 1);
					*rc_clone += 1;
					assert_eq!(*rc_clone, 2);
					channel2.send(());
				});

				channel.send(101);
				assert_eq!(channel2.recv().unwrap(), ());
				assert_eq!(*rc, 2);
			}
			assert_all_slabs_free();
			unsafe {
				cleanup_slab_allocators();
			}
		}
		assert_eq!(initial, unsafe { getalloccount() });
	}
}
