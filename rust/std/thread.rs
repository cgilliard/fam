use core::ops::{Drop, FnOnce};
use core::ptr;
use core::ptr::null_mut;
use prelude::*;
use sys::{map, thread_create, unmap};

struct ThreadWrap {
	wrap: *mut u64,
}

impl ThreadWrap {
	fn get<F>(&self) -> Box<F>
	where
		F: FnOnce(),
	{
		let v: *mut F = unsafe { *self.wrap } as *mut F;
		let metadata = unsafe { *(self.wrap.add(1) as *mut u64) };
		unsafe { Box::from_raw(v, metadata) }
	}
}

impl Drop for ThreadWrap {
	fn drop(&mut self) {
		unsafe {
			let handle = *(self.wrap.add(2) as *mut u64);
			unmap(handle as *mut u8, 1);
			unmap(self.wrap as *mut u8, 1);
		}
	}
}

extern "C" fn start_thread<F>(wrap: *mut u8) -> *mut u8
where
	F: FnOnce(),
{
	let wrap = wrap as *mut u64;
	let tw = ThreadWrap { wrap };
	let closure_box = tw.get::<F>();
	let closure = unsafe { closure_box.into_inner() };

	unsafe {
		ptr::read(closure)();
	}

	null_mut()
}

pub fn spawn<F>(f: F) -> Result<(), Error>
where
	F: FnOnce(),
{
	let mut b = Box::new(f).unwrap();
	let wrap = unsafe { map(1) } as *mut u64;
	let handle = unsafe { map(1) };
	unsafe {
		let v: u64 = b.as_mut_ptr() as u64;
		*(wrap.add(0)) = v;
		*(wrap.add(1)) = b.metadata();
		*(wrap.add(2)) = handle as u64;
		b.leak();
		thread_create(handle, start_thread::<F>, wrap as *mut u8, true);
	}

	Ok(())
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
					if *rc == 1 {
					} else {
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
