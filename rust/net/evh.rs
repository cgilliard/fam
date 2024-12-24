use core::ops::FnMut;
use prelude::*;

struct WriteHandleInner {
	wbuf: Vec<u8>,
}

pub struct WriteHandle {
	inner: Rc<WriteHandleInner>,
}

pub struct Connection {
	id: u64,
	handle: [u8; 4],
	inner: WriteHandle,
	rbuf: Vec<u8>,
}

pub trait EventHandler {
	fn start(&mut self) -> Result<(), Error>;
	fn set_on_read(
		&mut self,
		handler: &dyn FnMut(Connection) -> Result<(), Error>,
	) -> Result<(), Error>;
	fn set_on_accept(&mut self) -> Result<(), Error>;
	fn set_on_close(&mut self) -> Result<(), Error>;
	fn housekeeper(&mut self) -> Result<(), Error>;
	fn add_server(&mut self) -> Result<(), Error>;
	fn add_client(&mut self) -> Result<(), Error>;
}

struct EventHandlerImpl {}
