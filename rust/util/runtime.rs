use core::default::Default;
use core::marker::PhantomData;
use core::ops::FnMut;
use prelude::*;

type Task<T> = Box<dyn FnMut() -> T>;

pub struct RuntimeConfig {
	min_threads: u64,
	max_threads: u64,
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

struct State {
	waiting_workers: u64,
	total_workers: u64,
	halt: bool,
}

enum Message<T> {
	Task((Task<T>, Channel<T>, Rc<bool>)),
	Halt(()),
}

struct RuntimeImpl<T> {
	channel: Channel<Message<T>>,
	state: Rc<State>,
	lock: LockBox,
	stop_channel: Channel<()>,
}

impl<T> RuntimeImpl<T> {
	fn new(total_workers: u64) -> Result<Self, Error> {
		let lock = match lock_box!() {
			Ok(lock) => lock,
			Err(e) => return Err(e),
		};
		let state = match Rc::new(State {
			waiting_workers: 0,
			total_workers,
			halt: false,
		}) {
			Ok(state) => state,
			Err(e) => return Err(e),
		};
		let stop_channel = match Channel::new(total_workers) {
			Ok(stop_channel) => stop_channel,
			Err(e) => return Err(e),
		};
		match Channel::new(total_workers) {
			Ok(channel) => Ok(Self {
				stop_channel,
				channel,
				state,
				lock,
			}),
			Err(e) => Err(e),
		}
	}
}

pub struct Runtime<T> {
	config: RuntimeConfig,
	rimpl: Option<RuntimeImpl<T>>,
}

impl<T> Drop for Runtime<T> {
	fn drop(&mut self) {
		let _ = self.stop();
	}
}

impl<T> Handle<T> {
	pub fn block_on(&self) -> Result<T, Error> {
		self.channel.recv()
	}

	pub fn is_complete(&self) -> bool {
		*self.is_complete
	}
}

impl<T> Runtime<T> {
	pub fn new(config: RuntimeConfig) -> Result<Self, Error> {
		if config.max_threads == 0 || config.min_threads > config.max_threads {
			return Err(err!(IllegalArgument));
		}

		Ok(Self {
			config,
			rimpl: None,
		})
	}

	pub fn start(&mut self) -> Result<(), Error> {
		let rimpl = match RuntimeImpl::new(self.config.min_threads) {
			Ok(rimpl) => rimpl,
			Err(e) => return Err(e),
		};
		self.rimpl = Some(rimpl);
		for _i in 0..self.config.min_threads {
			let _ = self.thread();
		}
		Ok(())
	}

	pub fn stop(&mut self) -> Result<(), Error> {
		match &mut self.rimpl {
			Some(rimpl) => {
				{
					let _v = rimpl.lock.write();
					rimpl.state.halt = true;
				}

				loop {
					let _v = rimpl.lock.read();
					if rimpl.state.total_workers > 0 {
						let _ = rimpl.channel.send(Message::Halt(()));
					} else {
						break;
					}
				}
				let _ = rimpl.stop_channel.recv();
				Ok(())
			}
			None => Err(err!(NotInitialized)),
		}
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
			None => return Err(err!(NotInitialized)),
		};

		let channel = match Channel::new(self.config.max_threads) {
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

	pub fn cur_threads(&self) -> u64 {
		match &self.rimpl {
			Some(rimpl) => {
				let _v = rimpl.lock.read();
				rimpl.state.total_workers
			}
			None => 0,
		}
	}

	pub fn idle_threads(&self) -> u64 {
		match &self.rimpl {
			Some(rimpl) => {
				let _v = rimpl.lock.read();
				rimpl.state.waiting_workers
			}
			None => 0,
		}
	}

	fn thread(&mut self) -> Result<(), Error> {
		// SAFETY: must not be None because we return error in last step if so.
		let rimpl = self.rimpl.as_mut().unwrap();
		let channel = match rimpl.channel.clone() {
			Ok(channel) => channel,
			Err(e) => return Err(e),
		};

		let mut state: Rc<State> = match rimpl.state.clone() {
			Ok(state) => state,
			Err(e) => return Err(e),
		};
		let lock = match rimpl.lock.clone() {
			Ok(lock) => lock,
			Err(e) => return Err(e),
		};
		let _ = spawnj(move || loop {
			{
				let _l = lock.write();
				state.waiting_workers += 1;
				if state.waiting_workers > self.config.min_threads || state.halt {
					state.total_workers -= 1;
					state.waiting_workers -= 1;
					if state.halt && state.total_workers == 0 {
						let rimpl = self.rimpl.as_mut().unwrap();
						let _ = rimpl.stop_channel.send(());
					}
					return;
				}
			}

			let task = match channel.recv() {
				Ok(msg) => msg,
				Err(_e) => {
					return;
				}
			};

			{
				let _l = lock.write();
				state.waiting_workers -= 1;

				if state.waiting_workers == 0 && state.total_workers < self.config.max_threads {
					state.total_workers += 1;
					match self.thread() {
						Ok(_) => {}
						Err(_e) => {
							state.total_workers -= 1;
							println!("err!");
						}
					}
				}
			}

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
				Message::Halt(_) => {}
			}
		});
		Ok(())
	}
}

#[cfg(test)]
mod test {
	/*
	use super::*;
	use sys::getalloccount;

	#[test]
	fn test_runtime() {
		let mut r = Runtime::new(RuntimeConfig::default()).unwrap();
		r.start().unwrap();

		let v = 1;

		let (sender, receiver) = channel!(10).unwrap();
		let lock = lock_box!().unwrap();
		let lock_clone = lock.clone().unwrap();
		let (sender2, receiver2) = channel!(10).unwrap();
		let mut rc = Rc::new(0).unwrap();
		let mut rc_clone = rc.clone().unwrap();
		let rc_confirm = rc.clone().unwrap();

		let x1 = r
			.execute(move || -> Result<i32, Error> {
				let x = receiver.recv().unwrap();
				assert_eq!(x, 60);
				let _v = lock.write();
				*rc += 1;
				Ok(v + 40)
			})
			.unwrap();

		let x2 = r
			.execute(move || -> Result<i32, Error> {
				let x = receiver2.recv().unwrap();
				assert_eq!(x, 70);
				let _v = lock_clone.write();
				*rc_clone += 1;
				Ok(v + 4)
			})
			.unwrap();

		assert!(!x1.is_complete());
		assert!(!x2.is_complete());

		sender.send(60).unwrap();
		sender2.send(70).unwrap();

		assert_eq!(x1.block_on().unwrap().unwrap(), 41);
		assert_eq!(x2.block_on().unwrap().unwrap(), 5);
		assert!(x1.is_complete());
		assert!(x2.is_complete());
		assert_eq!(*rc_confirm, 2);
	}

	#[test]
	fn test_runtime_memory() {
		let initial = unsafe { getalloccount() };
		{
			{
				let mut r = Runtime::new(RuntimeConfig::default()).unwrap();
				r.start().unwrap();

				let v = 1;

				let (sender, receiver) = channel!(10).unwrap();
				let lock = lock_box!().unwrap();
				let lock_clone = lock.clone().unwrap();
				let (sender2, receiver2) = channel!(10).unwrap();
				let mut rc = Rc::new(0).unwrap();
				let mut rc_clone = rc.clone().unwrap();
				let rc_confirm = rc.clone().unwrap();

				let x1 = r
					.execute(move || -> Result<i32, Error> {
						let x = receiver.recv().unwrap();
						assert_eq!(x, 60);
						let _v = lock.write();
						*rc += 1;
						Ok(v + 40)
					})
					.unwrap();

				let x2 = r
					.execute(move || -> Result<i32, Error> {
						let x = receiver2.recv().unwrap();
						assert_eq!(x, 70);
						let _v = lock_clone.write();
						*rc_clone += 1;
						Ok(v + 4)
					})
					.unwrap();

				assert!(!x1.is_complete());
				assert!(!x2.is_complete());

				sender.send(60).unwrap();
				sender2.send(70).unwrap();

				assert_eq!(x1.block_on().unwrap().unwrap(), 41);
				assert_eq!(x2.block_on().unwrap().unwrap(), 5);
				assert!(x1.is_complete());
				assert!(x2.is_complete());
				assert_eq!(*rc_confirm, 2);
			}
		}
		assert_eq!(initial, unsafe { getalloccount() });
	}

	#[test]
	fn test_thread_pool_size() {
		let initial = unsafe { getalloccount() };
		{
			let mut r = Runtime::new(RuntimeConfig {
				min_threads: 2,
				max_threads: 4,
			})
			.unwrap();
			r.start().unwrap();

			while r.idle_threads() != 2 {}

			let (senda1, recva1) = channel!(10).unwrap();
			let (sendb1, recvb1) = channel!(10).unwrap();
			let (sendc1, recvc1) = channel!(10).unwrap();

			let x1 = r
				.execute(move || -> Result<i32, Error> {
					assert_eq!(recva1.recv().unwrap(), 1);
					sendb1.send(1).unwrap();
					assert_eq!(recvc1.recv().unwrap(), 1);
					Ok(1)
				})
				.unwrap();

			let (senda2, recva2) = channel!(10).unwrap();
			let (sendb2, recvb2) = channel!(10).unwrap();
			let (sendc2, recvc2) = channel!(10).unwrap();

			let x2 = r
				.execute(move || -> Result<i32, Error> {
					assert_eq!(recva2.recv().unwrap(), 2);
					sendb2.send(2).unwrap();
					assert_eq!(recvc2.recv().unwrap(), 2);
					Ok(2)
				})
				.unwrap();

			senda1.send(1).unwrap();
			senda2.send(2).unwrap();

			assert_eq!(recvb1.recv().unwrap(), 1);
			assert_eq!(recvb2.recv().unwrap(), 2);

			// we know there should be three threads spawned at this point because at least one
			// waiting worker is maintained.
			assert_eq!(r.cur_threads(), 3);

			sendc1.send(1).unwrap();
			sendc2.send(2).unwrap();

			assert_eq!(x1.block_on().unwrap().unwrap(), 1);
			assert_eq!(x2.block_on().unwrap().unwrap(), 2);

			while r.cur_threads() != 2 {}

			// The other two threads have exited so we should be back down to our min
			assert_eq!(r.cur_threads(), 2);

			// now start up 5 threads (we'll hit our limit of 4)
			let (senda1, recva1) = channel!(10).unwrap();
			let (sendb1, recvb1) = channel!(10).unwrap();
			let (sendc1, recvc1) = channel!(10).unwrap();

			let x1 = r
				.execute(move || -> Result<i32, Error> {
					assert_eq!(recva1.recv().unwrap(), 1);
					sendb1.send(1).unwrap();
					assert_eq!(recvc1.recv().unwrap(), 1);
					Ok(1)
				})
				.unwrap();

			let (senda2, recva2) = channel!(10).unwrap();
			let (sendb2, recvb2) = channel!(10).unwrap();
			let (sendc2, recvc2) = channel!(10).unwrap();

			let x2 = r
				.execute(move || -> Result<i32, Error> {
					assert_eq!(recva2.recv().unwrap(), 2);
					sendb2.send(2).unwrap();
					assert_eq!(recvc2.recv().unwrap(), 2);
					Ok(2)
				})
				.unwrap();

			let (senda3, recva3) = channel!(10).unwrap();
			let (sendb3, recvb3) = channel!(10).unwrap();
			let (sendc3, recvc3) = channel!(10).unwrap();

			let x3 = r
				.execute(move || -> Result<i32, Error> {
					assert_eq!(recva3.recv().unwrap(), 3);
					sendb3.send(3).unwrap();
					assert_eq!(recvc3.recv().unwrap(), 3);
					Ok(3)
				})
				.unwrap();

			let (senda4, recva4) = channel!(10).unwrap();
			let (sendb4, recvb4) = channel!(10).unwrap();
			let (sendc4, recvc4) = channel!(10).unwrap();

			let x4 = r
				.execute(move || -> Result<i32, Error> {
					assert_eq!(recva4.recv().unwrap(), 4);
					sendb4.send(4).unwrap();
					assert_eq!(recvc4.recv().unwrap(), 4);
					Ok(4)
				})
				.unwrap();

			let (senda5, recva5) = channel!(10).unwrap();
			let (sendb5, recvb5) = channel!(10).unwrap();
			let (sendc5, recvc5) = channel!(10).unwrap();

			let x5 = r
				.execute(move || -> Result<i32, Error> {
					assert_eq!(recva5.recv().unwrap(), 5);
					sendb5.send(5).unwrap();
					assert_eq!(recvc5.recv().unwrap(), 5);
					Ok(5)
				})
				.unwrap();

			senda1.send(1).unwrap();
			senda2.send(2).unwrap();
			senda3.send(3).unwrap();
			senda4.send(4).unwrap();

			assert_eq!(recvb1.recv().unwrap(), 1);
			assert_eq!(recvb2.recv().unwrap(), 2);
			assert_eq!(recvb3.recv().unwrap(), 3);
			assert_eq!(recvb4.recv().unwrap(), 4);

			// we are now at our max threads (4) there would have been a 5th, but we hit the
			// max.
			assert_eq!(r.cur_threads(), 4);

			// send messages to release all threads
			senda5.send(5).unwrap();
			sendc1.send(1).unwrap();
			sendc2.send(2).unwrap();
			sendc3.send(3).unwrap();
			sendc4.send(4).unwrap();
			sendc5.send(5).unwrap();

			// thread 5 can now complete
			assert_eq!(recvb5.recv().unwrap(), 5);

			while r.cur_threads() != 2 {}

			// After things settle down we should return to our min thread level of 2
			assert_eq!(r.cur_threads(), 2);

			assert_eq!(x1.block_on().unwrap().unwrap(), 1);
			assert_eq!(x2.block_on().unwrap().unwrap(), 2);
			assert_eq!(x3.block_on().unwrap().unwrap(), 3);
			assert_eq!(x4.block_on().unwrap().unwrap(), 4);
			assert_eq!(x5.block_on().unwrap().unwrap(), 5);
		}
		assert_eq!(initial, unsafe { getalloccount() });
	}
		*/
}
