#[macro_export]
macro_rules! err {
	($v:expr) => {
		Error { kind: $v }
	};
}

#[derive(PartialEq)]
pub enum ErrorKind {
	NoError = 0,
	Alloc = 1,
	OutOfBounds = 2,
}

pub struct Error {
	pub kind: ErrorKind,
}
