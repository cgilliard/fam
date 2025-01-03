use core::ops::FnOnce;
use core::ptr;
use prelude::*;
use sys::{
	safe_release, safe_thread_create, safe_thread_create_joinable, safe_thread_detach,
	safe_thread_handle_size, safe_thread_join,
};

pub struct JoinHandle {
	handle: [u8; 8],
	need_detach: bool,
}

impl Drop for JoinHandle {
	fn drop(&mut self) {
		if self.need_detach {
			let _x = self.detach();
		}
	}
}

impl JoinHandle {
	pub fn join(&mut self) -> Result<(), Error> {
		if !self.need_detach {
			Err(err!(ThreadJoin))
		} else if safe_thread_join(&self.handle as *const u8) != 0 {
			Err(err!(ThreadJoin))
		} else {
			self.need_detach = false;
			Ok(())
		}
	}

	pub fn detach(&mut self) -> Result<(), Error> {
		if !self.need_detach || safe_thread_detach(&self.handle as *const u8) != 0 {
			Err(err!(ThreadDetach))
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
	let closure = unsafe {
		let mut closure_box = Box::from_raw(Ptr::new(ptr as *mut F));
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
				return Err(err!(ThreadCreate));
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
	if safe_thread_handle_size() > 8 {
		exit!(
			"safe_thread_handle_size() > 8 ({})",
			safe_thread_handle_size()
		);
	}
	let jh = JoinHandle {
		handle: [0u8; 8],
		need_detach: true,
	};
	match Box::new(f) {
		Ok(mut b) => {
			if safe_thread_create_joinable(
				&jh.handle as *const u8,
				start_thread::<F>,
				b.as_ptr().raw() as *mut u8,
			) != 0
			{
				return Err(err!(ThreadCreate));
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
	use sys::safe_getalloccount;
	use sys::sleep_millis;

	#[test]
	fn test_threads() {
		let initial = safe_getalloccount();
		{
			let lock = lock!();
			let mut x = 1u32;
			let rc = Rc::new(1).unwrap();
			let mut rc_clone = rc.clone().unwrap();
			let mut jh = spawnj(|| {
				let _v = lock.write();
				x += 1;
				assert_eq!(x, 2);
				assert_eq!(*rc_clone, 1);
				*rc_clone += 1;
				assert_eq!(*rc_clone, 2);
			})
			.unwrap();

			loop {
				let _v = lock.write();
				if *rc != 1 {
					assert_eq!(*rc, 2);
					assert_eq!(x, 2);
					break;
				}
			}

			assert!(jh.join().is_ok());
		}
		assert_eq!(initial, safe_getalloccount());
	}
	#[test]
	fn test_threads2() {
		let initial = safe_getalloccount();
		{
			let lock = lock!();
			let mut x = 1u32;
			let mut jh = spawnj(|| {
				let _v = lock.write();
				crate::sys::safe_sleep_millis(50);
				x += 1;
				assert_eq!(x, 2);
			})
			.unwrap();

			loop {
				let _v = lock.write();
				if x != 1 {
					assert_eq!(x, 2);
					break;
				}
			}

			assert!(jh.join().is_ok());
		}
		assert_eq!(initial, safe_getalloccount());
	}

	#[test]
	fn test_thread_join() {
		let initial = safe_getalloccount();
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
		assert_eq!(initial, safe_getalloccount());
	}
}
