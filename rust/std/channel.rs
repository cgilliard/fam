/*
use core::marker::PhantomData;
use core::mem::size_of;
use core::ops::Drop;
use core::ptr;
use core::ptr::null_mut;
use prelude::*;
use std::boxed::get_slab_allocator;
use std::slabs::Slab;
use sys::{channel_handle_size, channel_init, channel_recv, channel_send, map, unmap, Message};

pub struct Channel<T> {
	handle_ptr: *mut u8,
	_marker: PhantomData<T>,
}

struct Payload {
	bptr: *mut u8,
	metadata: u64,
	payloadptr: *mut u8,
	payloadid: usize,
	msgptr: *mut u8,
	msgid: usize,
}

impl<T> Drop for Channel<T> {
	fn drop(&mut self) {
		unsafe {
			unmap(self.handle_ptr, 1);
		}
	}
}

impl<T> Channel<T> {
	pub fn new() -> Result<Channel<T>, Error> {
		if unsafe { channel_handle_size() } > 128 {
			exit!("channel handle is too big!");
		}
		let ret = Channel {
			handle_ptr: unsafe { map(1) },
			_marker: PhantomData,
		};
		unsafe {
			channel_init(ret.handle_ptr);
		}

		Ok(ret)
	}

	pub fn send(&mut self, t: T) -> Result<(), Error> {
		print!("slabsizes: ");
		print_num!(size_of::<Payload>());
		print!(" ");
		print_num!(size_of::<Message>());
		println!("");
		let payloadsa = match get_slab_allocator(size_of::<Payload>()) {
			Some(sa) => sa,
			None => return Err(ErrorKind::Alloc.into()),
		};
		let msgsa = match get_slab_allocator(size_of::<Message>()) {
			Some(sa) => sa,
			None => return Err(ErrorKind::Alloc.into()),
		};
		println!("box new");
		let mut b = match Box::new(t) {
			Ok(b) => b,
			Err(e) => return Err(e),
		};
		println!("alloc1");
		let mut msgslab = match msgsa.alloc() {
			Ok(slab) => slab,
			Err(e) => return Err(e),
		};
		println!("alloc2");
		let payloadslab = match payloadsa.alloc() {
			Ok(slab) => slab,
			Err(e) => {
				msgsa.free(&mut msgslab);
				return Err(e);
			}
		};

		print!("msg: ");
		print_num!(msgslab.get_id());
		print!(" ");
		print_num!(msgslab.get_raw());
		println!("");
		print!("payload: ");
		print_num!(payloadslab.get_id());
		print!(" ");
		print_num!(payloadslab.get_raw());
		println!("");

		unsafe {
			ptr::write(
				payloadslab.get_raw() as *mut Payload,
				Payload {
					bptr: b.as_mut_ptr() as *mut u8,
					metadata: b.metadata(),
					payloadptr: payloadslab.get_raw(),
					payloadid: payloadslab.get_id(),
					msgptr: msgslab.get_raw(),
					msgid: msgslab.get_id(),
				},
			);

			ptr::write(
				msgslab.get_raw() as *mut Message,
				Message {
					_next: null_mut(),
					payload: payloadslab.get_raw(),
				},
			);
			println!("write complete");
			//let handle_ptr: *mut u8 = &mut self.handle as *mut u8;
			channel_send(self.handle_ptr, msgslab.get_raw());
			println!("send complete");
			b.leak();
		}
		Ok(())
	}

	pub fn recv(&self) -> Result<T, Error> {
		let payloadsa = match get_slab_allocator(size_of::<Payload>()) {
			Some(sa) => sa,
			None => exit!("expected slab allocator not found!"),
		};

		let msgsa = match get_slab_allocator(size_of::<Message>()) {
			Some(sa) => sa,
			None => exit!("expected slab allocator not found!"),
		};

		unsafe {
			let recv = channel_recv(self.handle_ptr) as *const Message;
			let payload = (*recv).payload as *mut Payload;

			let ptr = (*payload).bptr;
			let metadata = (*payload).metadata;
			let nbox = Box::from_raw(ptr, metadata);
			let ret = ptr::read(nbox.into_inner() as *mut T);

			print!("freepayload: ");
			print_num!((*payload).payloadid);
			print!(" ");
			print_num!((*payload).payloadptr);
			println!("");

			print!("freemsgid: ");
			print_num!((*payload).msgid);
			print!(" ");
			print_num!((*payload).msgptr);
			println!("");

			let mut msgslab = Slab::from_raw((*payload).msgptr, (*payload).msgid);
			let mut payloadslab = Slab::from_raw((*payload).payloadptr, (*payload).payloadid);

			payloadsa.free(&mut payloadslab);
			msgsa.free(&mut msgslab);

			Ok(ret)
		}
	}
}
*/

/*
  use core::marker::PhantomData;
use core::mem::size_of;
use core::ops::Drop;
use core::ptr;
use core::ptr::null_mut;
use prelude::*;
use std::boxed::get_slab_allocator;
use std::slabs::Slab;
use sys::{channel_handle_size, channel_init, channel_recv, channel_send, map, unmap, Message};

pub struct Channel<T> {
	handle_data: [u8; 128],
	_marker: PhantomData<T>,
}

struct Payload<T> {
	bptr: *mut T,
	metadata: u64,
	selfptr: *mut u8,
	id: usize,
}

struct MessageHolder {
	msg: Message,
	selfptr: *mut u8,
	id: usize,
}

impl<T> Drop for Channel<T> {
	fn drop(&mut self) {}
}

impl<T> Channel<T> {
	pub fn new() -> Result<Channel<T>, Error> {
		if unsafe { channel_handle_size() } > 128 {
			exit!("channel handle is too big!");
		}
		unsafe {
			let handle_data = [0u8; 128];

			let ret = Channel {
				handle_data,
				_marker: PhantomData,
			};

			let handle_ptr: *const u8 = &ret.handle_data as *const u8;
			channel_init(handle_ptr);

			Ok(ret)
		}
	}

	pub fn send(&self, t: T) -> Result<(), Error> {
		let sa = match get_slab_allocator(size_of::<Payload<T>>()) {
			Some(sa) => sa,
			None => return Err(ErrorKind::Alloc.into()),
		};
		let msgsa = match get_slab_allocator(size_of::<MessageHolder>()) {
			Some(sa) => sa,
			None => return Err(ErrorKind::Alloc.into()),
		};
		let mut b = match Box::new(t) {
			Ok(b) => b,
			Err(e) => return Err(e),
		};
		unsafe {
			//let msg = map(1) as *mut Message;

			let mut msgslab = match msgsa.alloc() {
				Ok(slab) => slab,
				Err(e) => return Err(e),
			};

			// TODO: handle deallocation
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
			println!("ok");

			//(*msg).payload = slab.get_raw() as *mut u8;
			ptr::write(
				msgslab.get_raw() as *mut MessageHolder,
				MessageHolder {
					msg: Message {
						_next: null_mut(),
						payload: slab.get_raw() as *mut u8,
					},
					selfptr: msgslab.get_raw(),
					id: msgslab.get_id(),
				},
			);

			//(*(*msgslab.get_raw() as *mut MessageHolder)).msg.payload = slab.get_raw() as *mut u8;
			println!("2");

			let handle_ptr: *const u8 = &self.handle_data as *const u8;
			channel_send(handle_ptr, msgslab.get_raw() as *const u8);

			b.leak();
			//msgsa.free(&mut msgslab);
			Ok(())
		}
	}

	pub fn recv(&self) -> Result<T, Error> {
		let sa = match get_slab_allocator(size_of::<Payload<T>>()) {
			Some(sa) => sa,
			None => exit!("expected slab allocator not found!"),
		};
		unsafe {
			let handle_ptr: *const u8 = &self.handle_data as *const u8;
			let recv = channel_recv(handle_ptr) as *mut Message;
			println!("recv");
			if recv.is_null() {
				return Err(ErrorKind::ChannelRecv.into());
			}
			let payload = (*recv).payload as *mut Payload<T>;
			let ptr = (*payload).bptr;
			let metadata = (*payload).metadata;
			let nbox = Box::from_raw(ptr, metadata);
			let mut slab = Slab::from_raw((*payload).selfptr, (*payload).id);
			sa.free(&mut slab);

			//unmap(recv as *mut u8, 1);
			Ok(ptr::read(nbox.into_inner()))
		}
	}
}
*/

/*
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
				let mut channel = Channel::new().unwrap();
				let mut channel2 = Channel::new().unwrap();
				let rc = Rc::new(1).unwrap();
				let mut rc_clone = rc.clone().unwrap();
				spawn(|| {
					let v = channel.recv().unwrap();
					assert_eq!(v, 101);
					assert_eq!(*rc_clone, 1);
					*rc_clone += 1;
					assert_eq!(*rc_clone, 2);
					channel2.send(0);
				});

				channel.send(101);
				assert_eq!(channel2.recv().unwrap(), 0);
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
*/
