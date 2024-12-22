use core::default::Default;
use core::ops::FnMut;
use prelude::*;

pub struct RuntimeConfig {
	min_threads: u32,
	max_threads: u32,
}

impl Default for RuntimeConfig {
	fn default() -> Self {
		Self {
			min_threads: 4,
			max_threads: 8,
		}
	}
}

pub struct JoinHandle {}

pub struct Runtime {}

impl Runtime {
	pub fn new(config: RuntimeConfig) -> Result<Self, Error> {
		if config.min_threads == 0 {}
		if config.max_threads == 0 {}
		Ok(Self {})
	}

	pub fn execute<F: FnMut(i32) -> i32>(_f: F) -> Result<JoinHandle, Error> {
		Err(ErrorKind::Todo.into())
	}
}
