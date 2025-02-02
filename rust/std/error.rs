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
	CorruptedData,
	IllegalArgument,
	CapacityExceeded,
	ThreadCreate,
	ThreadJoin,
	ThreadDetach,
	IllegalState,
	Overflow,
	NotInitialized,
	ChannelSend,
	ChannelInit,
	CreateFileDescriptor,
	ConnectionClosed,
	SecpInit,
	SecpErr,
	SecpOddParity,
	WsStop,
	MultiplexRegister,
	SocketConnect,
	Pipe,
	Connect,
	IO,
	Bind,
	InsufficientFunds,
	Todo,
});

#[derive(PartialEq)]
pub struct Error {
	pub kind: ErrorKind,
	pub line: u32,
	//	pub file: String,
	//	pub backtrace: String,
}

impl Error {
	pub fn new(kind: ErrorKind, line: u32, _file: &str) -> Self {
		//let backtrace;
		Self {
			//backtrace,
			kind,
			line,
			/*
			file: match String::new(file) {
				Ok(file) => file,
				Err(_) => String::empty(),
			},
						*/
		}
	}
}

impl Display for Error {
	fn format(&self, _f: &mut Formatter) -> Result<(), Error> {
		/*
		writeb!(
			*f,
			"Error[kind={},loc={}:{}]\n{}",
			self.kind.as_str(),
			self.file,
			self.line,
			self.backtrace
		)
			*/
		Ok(())
	}
}

#[cfg(test)]
mod test {
	use super::*;
	#[test]
	fn test_err() {
		let _x = err!(Alloc);
		//println!("x=\n'{}'\n", _x);
	}
}
