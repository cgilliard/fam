use core::default::Default;
use core::marker::{PhantomData, Send};
use core::ops::FnMut;
use prelude::*;

type Task<T> = Box<dyn FnMut() -> T + Send + 'static>;

pub struct RuntimeConfig {
	min_threads: usize,
	max_threads: usize,
}

impl Clone for RuntimeConfig {
	fn clone(&self) -> Result<Self, Error> {
		Ok(Self {
			min_threads: self.min_threads,
			max_threads: self.max_threads,
		})
	}
}

impl Default for RuntimeConfig {
	fn default() -> Self {
		Self {
			min_threads: 4,
			max_threads: 8,
		}
	}
}

pub struct Handle<T> {
	_marker: PhantomData<T>,
}

struct RuntimeImpl<T> {
	channel: Channel<Task<T>>,
}

impl<T> RuntimeImpl<T> {
	fn new() -> Result<Self, Error> {
		let channel = Channel::new().unwrap();
		Ok(Self { channel })
	}
}

pub struct Runtime<T> {
	config: RuntimeConfig,
	rimpl: Option<RuntimeImpl<T>>,
}

impl<T> Handle<T> {
	fn _block_on(&self) -> Result<T, Error> {
		Err(ErrorKind::Todo.into())
	}

	fn _is_complete(&self) -> bool {
		false
	}
}

impl<T> Runtime<T> {
	pub fn new(config: RuntimeConfig) -> Result<Self, Error> {
		if config.max_threads == 0 || config.min_threads > config.max_threads {
			return Err(ErrorKind::IllegalArgument.into());
		}

		Ok(Self {
			config,
			rimpl: None,
		})
	}

	pub fn start(&mut self) -> Result<(), Error> {
		let rimpl = match RuntimeImpl::new() {
			Ok(rimpl) => rimpl,
			Err(e) => return Err(e),
		};
		self.rimpl = Some(rimpl);
		for _i in 0..self.config.min_threads {
			let _ = self.thread();
		}
		Ok(())
	}

	pub fn execute<F>(&mut self, task: F) -> Result<Handle<u64>, Error>
	where
		F: FnMut() -> T + Send + 'static,
	{
		let task = Box::new(task).unwrap();
		let rimpl = match &self.rimpl {
			Some(rimpl) => rimpl,
			None => return Err(ErrorKind::NotInitialized.into()),
		};

		let _ = rimpl.channel.send(task);

		Ok(Handle {
			_marker: PhantomData,
		})
	}

	fn thread(&mut self) -> Result<(), Error> {
		// SAFETY: must not be None because we return error in last step if so.
		let channel = match self.rimpl.as_mut().unwrap().channel.clone() {
			Ok(channel) => channel,
			Err(e) => return Err(e),
		};
		let _ = spawnj(move || {
			let mut task = match channel.recv() {
				Ok(msg) => msg,
				Err(_e) => {
					return;
				}
			};
			(task)();
		});
		Ok(())
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn test_runtime() {
		let mut runtime = Runtime::new(RuntimeConfig::default()).unwrap();
		runtime.start().unwrap();

		unsafe {
			crate::sys::sleep_millis(10);
		}

		let _ = runtime.execute(|| -> i32 {
			//println!("exec");
			4
		});

		let _ = runtime.execute(|| -> i32 {
			//println!("next");
			4
		});

		unsafe {
			crate::sys::sleep_millis(10);
		}
	}
}
