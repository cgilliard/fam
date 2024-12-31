use core::default::Default;
use core::slice::from_raw_parts;
use prelude::*;

pub struct WsConfig {
	addr: [u8; 4],
	port: u16,
	backlog: i32,
	threads: usize,
	max_events: i32,
}

impl Default for WsConfig {
	fn default() -> Self {
		Self {
			addr: [127, 0, 0, 1],
			port: 0, // randomly selected port
			backlog: 10,
			threads: 4,
			max_events: 30,
		}
	}
}

pub struct WsMessage<'a> {
	msg: &'a [u8],
	path: String,
}

impl WsMessage<'_> {
	pub fn msg(&self) -> &[u8] {
		let len = self.msg.len();
		unsafe { from_raw_parts(self.msg.as_ptr(), len) }
	}

	pub fn path(&self) -> String {
		self.path.clone().unwrap()
	}
}

pub struct WriteHandle {
	buffer: Vec<u8>,
	lock: Lock,
}
