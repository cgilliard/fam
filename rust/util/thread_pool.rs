use crate::*;
use core::default::Default;
use core::ops::Fn;
use core::ptr::null_mut;
use sys::{map, thread_create, thread_detach, thread_handle_size, thread_join};

pub struct ThreadPoolConfig {
	max: u64,
	min: u64,
}

impl Default for ThreadPoolConfig {
	fn default() -> Self {
		Self { max: 8, min: 4 }
	}
}

pub trait JoinHandle {
	fn join(&mut self);
	fn detach(&mut self);
}

impl JoinHandle for JoinHandleImpl {
	fn join(&mut self) {}
	fn detach(&mut self) {}
}

pub struct JoinHandleImpl {}

pub trait ThreadPool {
	fn start(&self) -> Result<(), Error>;
	fn execute(&mut self, id: u64, closure: Box<dyn Fn(u64)>)
		-> Result<Box<dyn JoinHandle>, Error>;
}

pub struct ThreadPoolImpl {
	config: ThreadPoolConfig,
}

struct ThreadPoolImplIdPair<'a> {
	tp: &'a mut ThreadPoolImpl,
	id: u64,
	metadata: u64,
}

impl ThreadPool for ThreadPoolImpl {
	fn start(&self) -> Result<(), Error> {
		Ok(())
	}
	fn execute(
		&mut self,
		id: u64,
		closure: Box<dyn Fn(u64)>,
	) -> Result<Box<dyn JoinHandle>, Error> {
		closure(id);
		let mut arg = Box::new(ThreadPoolImplIdPair {
			tp: self,
			id,
			metadata: 0,
		})
		.unwrap();
		arg.metadata = arg.metadata();
		unsafe {
			arg.leak();
		}
		let handle = unsafe { map(1) };
		unsafe {
			thread_create(handle, Self::start_thread, arg.as_mut_ptr() as *mut u64);
			// ignore below this is just to avoid warnings for partial checkin
			thread_detach(handle);
			thread_join(handle);
			let _ = thread_handle_size();
		}
		match Box::new(JoinHandleImpl {}) {
			Ok(b) => Ok(b),
			Err(e) => Err(e),
		}
	}
}

use core::ops::DerefMut;

impl ThreadPoolImpl {
	pub fn new(config: ThreadPoolConfig) -> Result<Box<dyn ThreadPool>, Error> {
		match Box::new(ThreadPoolImpl { config }) {
			Ok(b) => Ok(b),
			Err(e) => Err(e),
		}
	}

	pub fn begin_thread(&mut self, id: u64) {
		print!("begin thread: ");
		print_num!(id);
		print!(" ");
		print_num!(self.config.max + self.config.min);

		println!("");
	}

	extern "C" fn start_thread(arg: *mut u64) -> *mut u64 {
		let mut boxv: Box<ThreadPoolImplIdPair> =
			unsafe { Box::from_raw(arg as *mut ThreadPoolImplIdPair, 0) };
		unsafe {
			boxv.set_metadata(boxv.metadata);
		}
		println!("start thread");
		let id = boxv.id;
		Self::begin_thread(boxv.deref_mut().tp, id);

		null_mut()
	}
}

/*


#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn test_t1() {
		let config = ThreadPoolConfig::default();
		let mut t1 = ThreadPoolImpl::new(config).unwrap();
		t1.execute(
			90,
			Box::new(|x| {
				print!("num=");
				print_num!(x);
				println!("");
			})
			.unwrap(),
		);
	}
}
*/
