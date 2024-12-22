use core::default::Default;
use core::marker::{PhantomData, Send};
use core::ops::FnMut;
use prelude::*;

type Task = Box<dyn FnMut() -> u64 + Send + 'static>;

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

struct RuntimeImpl {
	channel: Channel<Task>,
}

impl RuntimeImpl {
	fn new() -> Result<Self, Error> {
		let channel = Channel::new().unwrap();
		Ok(Self { channel })
	}
}

pub struct Runtime {
	config: RuntimeConfig,
	rimpl: Option<RuntimeImpl>,
}

impl<T> Handle<T> {
	fn _block_on(&self) -> Result<T, Error> {
		Err(ErrorKind::Todo.into())
	}

	fn _is_complete(&self) -> bool {
		false
	}
}

impl Runtime {
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

	pub fn execute(&mut self, task: Task) -> Result<Handle<u64>, Error> {
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

		let _ = runtime.execute(
			Box::new(|| -> u64 {
				//println!("exec");
				4
			})
			.unwrap(),
		);

		let _ = runtime.execute(
			Box::new(|| -> u64 {
				//println!("next");
				4
			})
			.unwrap(),
		);

		unsafe {
			crate::sys::sleep_millis(10);
		}
	}
}
