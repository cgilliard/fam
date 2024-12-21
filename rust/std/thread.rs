use core::mem::size_of;
use core::ops::FnOnce;
use core::ptr;
use core::ptr::null_mut;
use prelude::*;
use std::boxed::get_slab_allocator;
use std::slabs::Slab;
use sys::thread_create;

struct ThreadInfo<F: FnOnce()> {
	ptr: *mut F,
	metadata: u64,
	selfptr: *mut u8,
	id: usize,
}

extern "C" fn start_thread<F>(wrap: *mut u8) -> *mut u8
where
	F: FnOnce(),
{
	let wrap = wrap as *mut ThreadInfo<F>;
	let (mut slab, closure) = unsafe {
		let ti = ptr::read(wrap);
		let v = ti.ptr;
		let metadata = ti.metadata;
		let closure_box = Box::from_raw(v, metadata);
		let closure = closure_box.into_inner();
		(Slab::from_raw(ti.selfptr, ti.id), ptr::read(closure))
	};

	let sa = match get_slab_allocator(size_of::<ThreadInfo<F>>()) {
		Some(sa) => sa,
		None => exit!("slab allocator not found!"),
	};
	sa.free(&mut slab);

	closure();

	null_mut()
}

pub fn spawn<F>(f: F) -> Result<(), Error>
where
	F: FnOnce(),
{
	match Box::new(f) {
		Ok(mut b) => {
			let sa = match get_slab_allocator(size_of::<ThreadInfo<F>>()) {
				Some(sa) => sa,
				None => return Err(ErrorKind::Alloc.into()),
			};

			let slab = match sa.alloc() {
				Ok(slab) => slab,
				Err(e) => return Err(e),
			};
			unsafe {
				ptr::write(
					slab.get_raw() as *mut ThreadInfo<F>,
					ThreadInfo {
						ptr: b.as_mut_ptr(),
						metadata: b.metadata(),
						selfptr: slab.get_raw(),
						id: slab.get_id(),
					},
				);

				b.leak();
			}

			let mut value: u128 = 0;
			let ptr: *mut u128 = &mut value as *mut u128;
			if unsafe { crate::sys::thread_handle_size() } > size_of::<u128>() {
				exit!("thread handle is too big!");
			}
			unsafe {
				if thread_create(
					ptr as *mut u8,
					start_thread::<F>,
					slab.get_raw() as *mut u8,
					true,
				) != 0
				{
					b.unleak();
					return Err(ErrorKind::ThreadCreate.into());
				}
			}

			Ok(())
		}
		Err(e) => Err(e),
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use std::boxed::assert_all_slabs_free;
	use sys::getalloccount;

	#[test]
	fn test_threads() {
		let initial = unsafe { getalloccount() };
		{
			{
				let lock = lock!();
				let mut x = 1;
				let rc = Rc::new(1).unwrap();
				let mut rc_clone = rc.clone().unwrap();
				spawn(|| {
					let _ = lock.read(); // memory fence only
					x += 1;
					assert_eq!(x, 2);
					assert_eq!(*rc_clone, 1);
					*rc_clone += 1;
					assert_eq!(*rc_clone, 2);
				});

				loop {
					let _ = lock.read(); // memory fence only
					if *rc != 1 {
						assert_eq!(*rc, 2);
						assert_eq!(x, 2);
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
