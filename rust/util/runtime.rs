use prelude::*;

type Task<T> = Box<dyn FnMut() -> T>;

pub struct RuntimeConfig {
	pub min_threads: u64,
	pub max_threads: u64,
}

pub struct Handle<T> {
	channel: Channel<T>,
	is_complete: Rc<bool>,
}

struct State {
	waiting_workers: u64,
	total_workers: u64,
	halt: bool,
}

enum Message<T> {
	Task((Task<T>, Channel<T>, Rc<bool>)),
	Halt(i32),
}

struct Runtime<T> {
	config: RuntimeConfig,
	channel: Channel<Message<T>>,
	state: Rc<State>,
	lock: LockBox,
	stop_channel: Channel<()>,
}

impl Default for RuntimeConfig {
	fn default() -> Self {
		Self {
			min_threads: 4,
			max_threads: 8,
		}
	}
}

impl<T> Drop for Runtime<T> {
	fn drop(&mut self) {
		let _ = self.stop();
	}
}

impl<T> Handle<T> {
	pub fn block_on(&self) -> T {
		self.channel.recv().unwrap()
	}

	pub fn is_complete(&self) -> bool {
		*self.is_complete
	}
}

impl<T> Runtime<T> {
	pub fn new(config: RuntimeConfig) -> Result<Self, Error> {
		let channel = Channel::new().unwrap();
		let state = Rc::new(State {
			waiting_workers: 0,
			total_workers: config.min_threads,
			halt: false,
		})
		.unwrap();
		let stop_channel = Channel::new().unwrap();
		let lock = lock_box!().unwrap();
		Ok(Self {
			config,
			channel,
			state,
			stop_channel,
			lock,
		})
	}

	pub fn start(&mut self) -> Result<(), Error> {
		{
			let _l = self.lock.read();
			if self.state.halt {
				return Err(err!(NotInitialized));
			}
		}
		for _i in 0..self.config.min_threads {
			match self.thread(self.config.min_threads, self.config.max_threads) {
				Ok(_) => {}
				Err(e) => return Err(e),
			}
		}
		Ok(())
	}

	pub fn stop(&mut self) -> Result<(), Error> {
		{
			let _l = self.lock.write();
			if self.state.halt {
				return Err(err!(NotInitialized));
			}
			self.state.halt = true;
			for _i in 0..self.config.max_threads {
				match self.channel.send(Message::Halt(1)) {
					Ok(_) => {}
					Err(e) => return Err(e),
				}
			}
		}
		self.stop_channel.recv().unwrap();

		Ok(())
	}

	pub fn execute<F>(&mut self, task: F) -> Result<Handle<T>, Error>
	where
		F: FnMut() -> T + 'static,
	{
		{
			let _l = self.lock.read();
			if self.state.halt {
				return Err(err!(NotInitialized));
			}
		}
		let channel = Channel::new().unwrap();
		let channel_clone = channel.clone().unwrap();
		let rc = Rc::new(false).unwrap();
		let rc_clone = rc.clone().unwrap();
		let task = Box::new(task).unwrap();
		let msg = Message::Task((task, channel, rc));
		match self.channel.send(msg) {
			Ok(_) => {}
			Err(e) => return Err(e),
		}
		Ok(Handle {
			channel: channel_clone,
			is_complete: rc_clone,
		})
	}

	pub fn cur_threads(&self) -> u64 {
		{
			let _l = self.lock.read();
			self.state.total_workers
		}
	}

	pub fn idle_threads(&self) -> u64 {
		{
			let _l = self.lock.read();
			self.state.waiting_workers
		}
	}

	#[inline(never)]
	fn thread(&mut self, min: u64, max: u64) -> Result<(), Error> {
		let channel = self.channel.clone().unwrap();
		let mut state = self.state.clone().unwrap();
		let lock = self.lock.clone().unwrap();
		spawn(move || {
			loop {
				{
					let _l = lock.write();
					if state.halt {
						state.total_workers -= 1;
						break;
					} else {
						state.waiting_workers += 1;
						if state.waiting_workers > min {
							state.total_workers -= 1;
							state.waiting_workers -= 1;
							break;
						}
					}
				}
				match channel.recv().unwrap() {
					Message::Task(mut t) => {
						{
							let _l = lock.write();
							state.waiting_workers -= 1;
							if state.waiting_workers == 0
								&& state.total_workers < max
								&& !state.halt
							{
								state.total_workers += 1;
								match self.thread(min, max) {
									Ok(_) => {}
									Err(e) => {
										println!("WARN: Could not start additional thread: ", e)
									}
								}
							}
						}
						let res = t.0();
						*t.2 = true;
						match t.1.send(res) {
							Ok(_) => {}
							Err(e) => {
								println!("WARN: could not send result: ", e);
							}
						}
					}
					Message::Halt(_x) => {}
				}
			}
			let _l = lock.read();
			if state.total_workers == 0 {
				match self.stop_channel.send(()) {
					Ok(_) => {}
					Err(e) => {
						println!("WARN: sending stop_channel generated error: {}", e);
					}
				}
			}
		})
		.unwrap();
		Ok(())
	}
}

#[cfg(test)]
mod test {
	use super::*;
	#[test]
	fn test_runtime1() {
		let initial = crate::sys::safe_getalloccount();
		{
			let mut x = Runtime::new(RuntimeConfig::default()).unwrap();
			assert!(x.start().is_ok());
			let (send1, recv1) = channel!().unwrap();
			let (send2, recv2) = channel!().unwrap();
			let handle1 = x
				.execute(move || -> i32 {
					assert_eq!(recv1.recv().unwrap(), 8);
					7
				})
				.unwrap();

			send1.send(8).unwrap();

			assert_eq!(handle1.block_on(), 7);
			assert!(handle1.is_complete());

			let handle2 = x
				.execute(move || -> i32 {
					send2.send(9).unwrap();
					6
				})
				.unwrap();

			assert_eq!(recv2.recv().unwrap(), 9);
			assert_eq!(handle2.block_on(), 6);
			assert!(handle2.is_complete());

			assert!(x.stop().is_ok());
		}
		assert_eq!(initial, crate::sys::safe_getalloccount());
	}

	#[test]
	fn test_runtime2() {
		let config = RuntimeConfig {
			min_threads: 5,
			max_threads: 5,
		};
		let mut x: Runtime<()> = Runtime::new(config).unwrap();
		assert!(x.start().is_ok());
		let (send1, recv1) = channel!().unwrap();
		let (send2, recv2) = channel!().unwrap();
		let (senda1, recva1) = channel!().unwrap();
		let (senda2, recva2) = channel!().unwrap();

		let h1 = x
			.execute(move || {
				send1.send(()).unwrap();
				recva1.recv().unwrap();
			})
			.unwrap();

		let h2 = x
			.execute(move || {
				send2.send(()).unwrap();
				recva2.recv().unwrap();
			})
			.unwrap();

		recv1.recv().unwrap();
		recv2.recv().unwrap();

		//	assert_eq!(x.cur_threads(), 3);
		//	assert_eq!(x.idle_threads(), 1);

		assert!(senda1.send(()).is_ok());
		assert!(senda2.send(()).is_ok());

		assert_eq!(h1.block_on(), ());
		assert_eq!(h2.block_on(), ());

		crate::sys::safe_sleep_millis(100);
		//	assert_eq!(x.cur_threads(), 2);
		//	assert_eq!(x.idle_threads(), 2);

		assert!(x.stop().is_ok());
	}
}
