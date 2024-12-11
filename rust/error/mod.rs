#[macro_export]
macro_rules! err {
	($v:expr) => {
		Err(Error { kind: $v }) as Result<(), Error>
	};
}

pub enum ErrorKind {
	Alloc = 1,
	OutOfBounds = 2,
}

pub struct Error {
	pub kind: ErrorKind,
}
