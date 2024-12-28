use core::ops::FnOnce;
use core::ptr;
use prelude::*;
use sys::{
	safe_alloc, safe_release, safe_thread_create, safe_thread_create_joinable, safe_thread_detach,
	safe_thread_handle_size, safe_thread_join,
};

pub struct JoinHandle {
	handle: *const u8,
	need_detach: bool,
}

impl Drop for JoinHandle {
	fn drop(&mut self) {
		if self.need_detach {
			let _ = self.detach();
		}
		safe_release(self.handle as *mut u8);
	}
}

impl JoinHandle {
	pub fn join(&mut self) -> Result<(), Error> {
		if !self.need_detach {
			Err(ErrorKind::ThreadJoin.into())
		} else if safe_thread_join(self.handle) != 0 {
			Err(ErrorKind::ThreadJoin.into())
		} else {
			self.need_detach = false;
			Ok(())
		}
	}

	pub fn detach(&mut self) -> Result<(), Error> {
		if !self.need_detach || safe_thread_detach(self.handle) != 0 {
			Err(ErrorKind::ThreadDetach.into())
		} else {
			self.need_detach = false;
			Ok(())
		}
	}
}

extern "C" fn start_thread<F>(ptr: *mut u8)
where
	F: FnOnce(),
{
	let mut closure_box: Box<F>;
	let closure = unsafe {
		closure_box = Box::from_raw(Ptr::new(ptr as *mut F));
		closure_box.leak();
		let closure = closure_box.as_ptr().raw() as *mut F;
		let ret = ptr::read(closure);
		safe_release(ptr);
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
			if safe_thread_create(start_thread::<F>, b.as_ptr().raw() as *mut u8) != 0 {
				return Err(ErrorKind::ThreadCreate.into());
			}
			b.leak();
			Ok(())
		}
		Err(e) => Err(e),
	}
}

pub fn spawnj<F>(f: F) -> Result<JoinHandle, Error>
where
	F: FnOnce(),
{
	let handle = safe_alloc(safe_thread_handle_size());
	let jh = JoinHandle {
		handle,
		need_detach: true,
	};
	match Box::new(f) {
		Ok(mut b) => {
			if safe_thread_create_joinable(
				jh.handle,
				start_thread::<F>,
				b.as_ptr().raw() as *mut u8,
			) != 0
			{
				return Err(ErrorKind::ThreadCreate.into());
			}
			b.leak();
			Ok(jh)
		}
		Err(e) => Err(e),
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use sys::getalloccount;
	use sys::sleep_millis;

	#[test]
	fn test_threads() {
		let initial = unsafe { getalloccount() };
		{
			let lock = lock!();
			let mut x = 1;
			let rc = Rc::new(1).unwrap();
			let mut rc_clone = rc.clone().unwrap();
			let mut jh = spawnj(|| {
				let _v = lock.read(); // memory fence only
				x += 1;
				assert_eq!(x, 2);
				assert_eq!(*rc_clone, 1);
				*rc_clone += 1;
				assert_eq!(*rc_clone, 2);
			})
			.unwrap();

			loop {
				let _v = lock.read(); // memory fence only
				if *rc != 1 {
					assert_eq!(*rc, 2);
					assert_eq!(x, 2);
					break;
				}
			}

			assert!(jh.join().is_ok());
		}
		assert_eq!(initial, unsafe { getalloccount() });
	}

	#[test]
	fn test_thread_join() {
		let initial = unsafe { getalloccount() };
		{
			let lock = lock!();
			let mut x = 1;
			let rc = Rc::new(1).unwrap();
			let mut rc_clone = rc.clone().unwrap();
			let mut jh = spawnj(|| {
				let _v = lock.read(); // memory fence only
				x += 1;
				assert_eq!(x, 2);
				assert_eq!(*rc_clone, 1);
				unsafe {
					sleep_millis(100);
				}
				*rc_clone += 1;
				assert_eq!(*rc_clone, 2);
			})
			.unwrap();

			assert!(jh.join().is_ok());
			assert_eq!(*rc, 2);
		}
		assert_eq!(initial, unsafe { getalloccount() });
	}
}
