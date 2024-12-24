use core::ops::FnMut;
use prelude::*;

struct WriteHandleInner {
	wbuf: Vec<u8>,
	handle: [u8; 4],
	id: u64,
	is_active: bool,
	lock: Lock,
}

pub struct WriteHandle {
	inner: Rc<WriteHandleInner>,
}

pub struct Connection {
	inner: WriteHandle,
	rbuf: Vec<u8>,
}

pub trait EventHandler {
	fn start(&mut self) -> Result<(), Error>;
	fn stop(&mut self) -> Result<(), Error>;
	fn set_on_read(
		&mut self,
		handler: &dyn FnMut(Connection) -> Result<(), Error>,
	) -> Result<(), Error>;
	fn set_on_accept(
		&mut self,
		handler: &dyn FnMut(Connection) -> Result<(), Error>,
	) -> Result<(), Error>;
	fn set_on_close(
		&mut self,
		handler: &dyn FnMut(Connection) -> Result<(), Error>,
	) -> Result<(), Error>;
	fn housekeeper(
		&mut self,
		handler: &dyn FnMut(Connection) -> Result<(), Error>,
	) -> Result<(), Error>;
	fn add_server(&mut self) -> Result<(), Error>;
	fn add_client(&mut self) -> Result<(), Error>;
}

struct EventHandlerImpl {}
