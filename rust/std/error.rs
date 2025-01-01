use core::fmt;
use core::fmt::Debug;
use core::result;
use prelude::*;

macro_rules! define_enum_with_strings {
    ($enum_name:ident { $($variant:ident),* $(,)? }) => {
        #[derive(PartialEq)]
        pub enum $enum_name {
            $($variant),*
        }

        impl $enum_name {
            pub fn as_str(&self) -> &'static str {
                match self {
                    $(Self::$variant => stringify!($variant),)*
                }
            }
        }
    };
}

// Define the enum and string conversion
define_enum_with_strings!(ErrorKind {
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
	ChannelInit,
	CreateFileDescriptor,
	MultiplexRegister,
	SocketConnect,
	IO,
	Todo,
});

#[derive(PartialEq)]
pub struct Error {
	pub kind: ErrorKind,
	pub line: u32,
	pub file: String,
}

impl Debug for Error {
	fn fmt(&self, _: &mut fmt::Formatter<'_>) -> result::Result<(), fmt::Error> {
		result::Result::Ok(())
	}
}

impl Error {
	pub fn new(kind: ErrorKind, line: u32, file: &str) -> Self {
		Self {
			kind,
			line,
			file: match String::new(file) {
				Ok(file) => file,
				Err(_) => String::empty(),
			},
		}
	}
}

impl Display for Error {
	fn format(&self, f: &mut Formatter) -> Result<(), Error> {
		writeb!(
			*f,
			"Error[kind={},loc={}:{}]",
			self.kind.as_str(),
			self.file,
			self.line
		)
	}
}
