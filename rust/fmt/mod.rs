use error::Error;
use result::Result;

pub struct Formatter {
	_buf: *mut u8,
}

pub trait Display {
	fn fmt(&self, f: &mut Formatter) -> Result<(), Error>;
}
