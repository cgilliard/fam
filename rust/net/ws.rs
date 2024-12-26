/*

use core::ops::FnMut;
use prelude::*;

pub struct WsConfig {
	host: String,
	port: u16,
}

pub struct WsRequest {}

pub struct WsResponse {}

pub struct WsServer {}

impl WsServer {
	pub fn new(
		&mut self,
		config: WsConfig,
		handler: &dyn FnMut(WsRequest, WsResponse),
	) -> Result<Self, Error> {
		Ok(Self {})
	}

	pub fn stop(&mut self) {
		|req: WsRequest, resp: WsResponse| {
			println!("ok");
		};
	}
}
*/
