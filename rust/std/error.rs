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
	IllegalState,
	NotInitialized,
	ChannelSend,
	ChannelInit,
	CreateFileDescriptor,
	ConnectionClosed,
	SecpInit,
	SecpErr,
	WsStop,
	MultiplexRegister,
	SocketConnect,
	Pipe,
	Connect,
	IO,
	Bind,
	Todo,
});

#[derive(PartialEq)]
pub struct Error {
	pub kind: ErrorKind,
	pub line: u32,
	pub file: String,
	pub backtrace: String,
}

impl Error {
	pub fn new(kind: ErrorKind, line: u32, file: &str) -> Self {
		let backtrace;
		#[cfg(test)]
		{
			use core::slice::from_raw_parts;
			use core::str::from_utf8_unchecked;
			use sys::{safe_backtrace_full, safe_cstring_len, safe_release};
			let s = "./bin/test_fam";
			let bt = safe_backtrace_full(s.as_ptr(), s.len());
			if bt.is_null() {
				backtrace = String::empty();
			} else {
				let len = safe_cstring_len(bt);
				let bt_slice = unsafe { from_raw_parts(bt, len) };
				let bt_str = unsafe { from_utf8_unchecked(bt_slice) };
				backtrace = match String::new(bt_str) {
					Ok(backtrace) => backtrace,
					Err(_) => String::empty(),
				};
				safe_release(bt);
			}
		}
		#[cfg(not(test))]
		{
			backtrace = String::empty();
		}
		Self {
			backtrace,
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
			"Error[kind={},loc={}:{}]\n{}",
			self.kind.as_str(),
			self.file,
			self.line,
			self.backtrace
		)
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
