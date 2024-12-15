#[macro_export]
macro_rules! err {
	($v:expr) => {
		Error { kind: $v }
	};
}

#[derive(PartialEq)]
pub enum ErrorKind {
	NoError,
	Alloc,
	OutOfBounds,
	IllegalArgument,
	CapacityExceeded,
	Todo,
}

pub struct Error {
	pub kind: ErrorKind,
}
