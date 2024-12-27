/*
use core::ops::FnMut;
use core::ptr::null_mut;
use prelude::*;
use sys::{alloc, release, socket_handle_size, socket_listen};

pub struct WsConfig {
	addr: [u8; 4],
	port: u16,
	backlog: i32,
	threads: usize,
}

impl Default for WsConfig {
	fn default() -> Self {
		Self {
			addr: [127, 0, 0, 1],
			port: 0, // randomly selected port
			backlog: 10,
			threads: 4,
		}
	}
}

pub struct WsRequest {}

pub struct WsResponse {}

pub struct WsServer {
	config: WsConfig,
	port: u16,
	handle: *mut u8,
}

impl WsServer {
	pub fn new(config: WsConfig) -> Result<Self, Error> {
		let port = config.port;
		let handle = null_mut();
		Ok(Self {
			config,
			port,
			handle,
		})
	}

	pub fn register_handler(
		&mut self,
		path: &str,
		handler: Box<dyn FnMut(WsRequest, WsResponse) -> Result<(), Error>>,
	) -> Result<(), Error> {
		Ok(())
	}

	pub fn start(&mut self) {
		unsafe {
			let handle = alloc(socket_handle_size());
			let port = socket_listen(
				handle,
				self.config.addr.as_ptr(),
				self.config.port,
				self.config.backlog,
			);
			self.port = port as u16;
			self.handle = handle;
			Self::start_threads(&self.config, handle);
		}
	}

	pub fn port(&self) -> u16 {
		self.port
	}

	pub fn stop(&mut self) {}

	fn start_threads(config: &WsConfig, handle: *mut u8) {
		for i in 0..config.threads {
			let _ = spawn(move || {
				println!("thread {}", i);
			});
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;
	#[test]
	fn test_ws1() {
		let config = WsConfig::default();
		let mut ws = WsServer::new(config).unwrap();
		ws.start();
		println!("port={}", ws.port());

		//park();
	}
}
*/
