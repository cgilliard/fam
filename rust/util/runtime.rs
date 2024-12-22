use core::default::Default;
use core::marker::PhantomData;
use core::ops::FnMut;
use prelude::*;

type Task<T> = Box<dyn FnMut() -> T>;

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
	channel: Channel<T>,
	is_complete: Rc<bool>,
	_marker: PhantomData<T>,
}

enum Message<T> {
	Task((Task<T>, Channel<T>, Rc<bool>)),
}

struct RuntimeImpl<T> {
	channel: Channel<Message<T>>,
}

impl<T> RuntimeImpl<T> {
	fn new() -> Result<Self, Error> {
		match Channel::new() {
			Ok(channel) => Ok(Self { channel }),
			Err(e) => Err(e),
		}
	}
}

pub struct Runtime<T> {
	config: RuntimeConfig,
	rimpl: Option<RuntimeImpl<T>>,
}

impl<T> Handle<T> {
	fn block_on(&self) -> Result<T, Error> {
		self.channel.recv()
	}

	fn is_complete(&self) -> bool {
		*self.is_complete
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

	pub fn execute<F>(&mut self, task: F) -> Result<Handle<T>, Error>
	where
		F: FnMut() -> T + 'static,
	{
		let task = match Box::new(task) {
			Ok(task) => task,
			Err(e) => return Err(e),
		};
		let rimpl = match &self.rimpl {
			Some(rimpl) => rimpl,
			None => return Err(ErrorKind::NotInitialized.into()),
		};

		let channel = match Channel::new() {
			Ok(channel) => channel,
			Err(e) => return Err(e),
		};
		let channel_clone = match channel.clone() {
			Ok(channel) => channel,
			Err(e) => return Err(e),
		};

		let is_complete = match Rc::new(false) {
			Ok(is_complete) => is_complete,
			Err(e) => return Err(e),
		};
		let is_complete_clone = match is_complete.clone() {
			Ok(is_complete_clone) => is_complete_clone,
			Err(e) => return Err(e),
		};
		match rimpl
			.channel
			.send(Message::Task((task, channel_clone, is_complete_clone)))
		{
			Ok(_) => Ok(Handle {
				channel,
				is_complete,
				_marker: PhantomData,
			}),
			Err(e) => Err(e),
		}
	}

	fn thread(&mut self) -> Result<(), Error> {
		// SAFETY: must not be None because we return error in last step if so.
		let channel = match self.rimpl.as_mut().unwrap().channel.clone() {
			Ok(channel) => channel,
			Err(e) => return Err(e),
		};
		let _ = spawnj(move || {
			let task = match channel.recv() {
				Ok(msg) => msg,
				Err(_e) => {
					return;
				}
			};

			match task {
				Message::Task(mut t) => {
					let ret = t.0();
					*t.2 = true;
					match t.1.send(ret) {
						Ok(_) => {}
						Err(_e) => {
							println!("err!");
						}
					}
				}
			}
		});
		Ok(())
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn test_runtime() {
		let mut r = Runtime::new(RuntimeConfig::default()).unwrap();
		r.start().unwrap();

		let v = 1;

		let (sender, receiver) = channel!().unwrap();
		let (lock, lock_clone) = lock_pair!().unwrap();
		let (sender2, receiver2) = channel!().unwrap();
		let (mut rc, mut rc_clone) = rc!(0).unwrap();
		let rc_confirm = rc.clone().unwrap();

		let x1 = r
			.execute(move || -> i32 {
				let x = receiver.recv().unwrap();
				assert_eq!(x, 60);
				let _ = lock.write();
				*rc += 1;
				v + 40
			})
			.unwrap();

		let x2 = r
			.execute(move || -> i32 {
				let x = receiver2.recv().unwrap();
				assert_eq!(x, 70);
				let _ = lock_clone.write();
				*rc_clone += 1;
				v + 4
			})
			.unwrap();

		assert!(!x1.is_complete());
		assert!(!x2.is_complete());

		sender.send(60).unwrap();
		sender2.send(70).unwrap();

		assert_eq!(x1.block_on().unwrap(), 41);
		assert_eq!(x2.block_on().unwrap(), 5);
		assert!(x1.is_complete());
		assert!(x2.is_complete());
		assert_eq!(*rc_confirm, 2);
	}
}
