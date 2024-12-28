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
	IO,
	Todo,
});

#[derive(PartialEq)]
pub struct Error {
	pub kind: ErrorKind,
}

impl Error {
	fn new(kind: ErrorKind) -> Self {
		Self { kind }
	}
}

impl From<ErrorKind> for Error {
	fn from(kind: ErrorKind) -> Error {
		Error::new(kind)
	}
}

impl Display for Error {
	fn format(&self, f: &mut Formatter) -> Result<(), Error> {
		writeb!(*f, "Error[kind={}]", self.kind.as_str())
	}
}
