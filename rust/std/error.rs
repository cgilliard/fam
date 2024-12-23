use prelude::*;

#[derive(PartialEq)]
pub enum ErrorKind {
	Unknown,
	Alloc,
	OutOfBounds,
	IllegalArgument,
	CapacityExceeded,
	ThreadCreate,
	ThreadJoin,
	ThreadDetach,
	NotInitialized,
	ChannelRecv,
	IO,
	Todo,
}

#[derive(PartialEq)]
pub struct Error {
	pub kind: ErrorKind,
}

impl From<ErrorKind> for Error {
	fn from(kind: ErrorKind) -> Error {
		Self { kind }
	}
}
