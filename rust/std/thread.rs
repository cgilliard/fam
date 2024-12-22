use core::ops::FnOnce;
use core::ptr;
use prelude::*;
use sys::thread_create;

extern "C" fn start_thread<F>(ptr: *mut u8)
where
	F: FnOnce(),
{
	let mut closure_box: Box<F>;
	let closure = unsafe {
		closure_box = Box::from_raw(ptr as *mut F);
		closure_box.leak();
		let closure = closure_box.into_inner() as *mut F;
		let ret = ptr::read(closure);
		crate::sys::release(ptr);
		ret
	};
	closure();
}

pub fn spawn<F>(f: F) -> Result<(), Error>
where
	F: FnOnce(),
{
	match Box::new(f) {
		Ok(mut b) => {
			unsafe {
				if thread_create(start_thread::<F>, b.as_mut_ptr() as *mut u8) != 0 {
					return Err(ErrorKind::ThreadCreate.into());
				}
				b.leak();
			}
			Ok(())
		}
		Err(e) => Err(e),
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use sys::getalloccount;

	#[test]
	fn test_threads() {
		let initial = unsafe { getalloccount() };
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
		assert_eq!(initial, unsafe { getalloccount() });
	}
}
