use core::ptr::copy_nonoverlapping;
use core::str::from_utf8_unchecked;
use prelude::*;
use sys::safe_f64_to_str;

#[macro_export]
macro_rules! writeb {
        ($f:expr, $fmt:expr) => {{
            writeb!($f, "{}", $fmt)
        }};
        ($f:expr, $fmt:expr, $($t:expr),*) => {{
            let mut err = ErrorKind::Unknown.into();
            match String::new($fmt) {
                Ok(fmt) => {
                    let mut cur = 0;
                    $(
                        match fmt.findn("{}", cur) {
                            Some(index) => {
                                    let s = fmt.substring( cur, cur + index).unwrap();
                                    let s = s.to_str();
                                    match $f.write_str(s, s.len()) {
                                        Ok(_) => {},
                                        Err(e) => err = e,
                                    }
                                    cur += index + 2;
                            },
                            None => {
                            },
                        }
                        match $t.format(&mut $f) {
                            Ok(_) => {},
                            Err(e) => err = e,
                        }
                    )*
                    let s = fmt.substring( cur, fmt.len()).unwrap();
                    let s = s.to_str();
                    match $f.write_str(s, s.len()) {
                        Ok(_) =>{},
                        Err(e) => err = e,
                    }
                }
                Err(e) => err = e,
            }

            if err.kind == ErrorKind::Unknown {
                Ok(())
            } else {
                Err(err)
            }
        }};
}

#[macro_export]
macro_rules! format {
        ($fmt:expr) => {{
                format!("{}", $fmt)
        }};
        ($fmt:expr, $($t:expr),*) => {{
                let mut formatter = Formatter::new();
                match writeb!(formatter, $fmt, $($t),*) {
                    Ok(_) => String::new(formatter.as_str()),
                    Err(e) => Err(e)
                }
        }};
}

#[macro_export]
macro_rules! exit {
        ($fmt:expr) => {{
                exit!("{}", $fmt);
        }};
        ($fmt:expr,  $($t:expr),*) => {{
                        use core::panic::Location;
                        use std::util::u128_to_str;
                        use sys::{safe_exit, safe_write};

                        safe_write(2, "Panic: ".as_ptr(), 7);
                        println!($fmt, $($t),*);
                        #[cfg(not(mrustc))]
                        {
                                let location = Location::caller();
                                let file = location.file();
                                let mut buf = [0u8; 32];
                                let len = u128_to_str(location.line() as u128, 0, &mut buf, 10);
                                safe_write(2, file.as_ptr(), file.len());
                                safe_write(2, ":".as_ptr(), 1);
                                safe_write(2, buf.as_ptr(), len);
                                safe_write(2, "\n".as_ptr(), 1);
                        }

                        safe_exit(-1);
                        loop {}
        }};
}

#[macro_export]
macro_rules! panic {
        ($fmt:expr) => {{
                exit!("{}", $fmt);
        }};
        ($fmt:expr,  $($t:expr),*) => {{
                exit!($fmt, $($t),*);
        }};
}

#[macro_export]
macro_rules! println {
    ($fmt:expr) => {{
            crate::sys::safe_write(2, $fmt.as_ptr(), $fmt.len());
            crate::sys::safe_write(2, "\n".as_ptr(), 1);
    }};
    ($fmt:expr, $($t:expr),*) => {{
        match format!($fmt, $($t),*) {
            Ok(line) => {
                crate::sys::safe_write(2, line.to_str().as_ptr(), line.len());
                crate::sys::safe_write(2, "\n".as_ptr(), 1);
            },
            Err(_e) => {},
        }
    }};
}

#[macro_export]
macro_rules! print {
    ($fmt:expr) => {{
        unsafe { crate::sys::write(2, $fmt.as_ptr(), $fmt.len()); }
    }};
    ($fmt:expr, $($t:expr),*) => {{
        match format!($fmt, $($t),*) {
            Ok(line) => {
                unsafe { crate::sys::write(2, line.to_str().as_ptr(), line.len()); }
            },
            Err(_e) => {},
        }
    }};
}

pub struct Formatter {
	buffer: Vec<u8>,
	pos: usize,
}

impl Formatter {
	pub fn new() -> Self {
		Self {
			buffer: Vec::new(),
			pos: 0,
		}
	}
	pub fn write_str(&mut self, s: &str, len: usize) -> Result<(), Error> {
		let bytes = s.as_bytes();
		match self.buffer.resize(len + self.pos) {
			Ok(_) => {}
			Err(e) => return Err(e),
		}

		unsafe {
			let ptr = (self.buffer.as_mut_ptr() as *mut u8).add(self.pos);
			copy_nonoverlapping(bytes.as_ptr(), ptr, len);
		}
		self.pos += len;

		Ok(())
	}
	pub fn as_str(&self) -> &str {
		if self.pos == 0 {
			""
		} else {
			unsafe { from_utf8_unchecked(&self.buffer[0..self.pos]) }
		}
	}
}

macro_rules! impl_display_unsigned {
    ($($t:ty),*) => {
        $(
            impl Display for $t {
                fn format(&self, f: &mut Formatter) -> Result<(), Error> {
                    let mut buf = [0u8; 64];
                    let len = u128_to_str((*self).into(), 0, &mut buf, 10);
                    unsafe { f.write_str(from_utf8_unchecked(&buf), len) }
                }
            }
        )*
    };
}

impl_display_unsigned!(u8, u16, u32, u64, u128);

impl Display for usize {
	fn format(&self, f: &mut Formatter) -> Result<(), Error> {
		let mut buf = [0u8; 64];
		let len = u128_to_str(*self as u128, 0, &mut buf, 10);
		unsafe { f.write_str(from_utf8_unchecked(&buf), len) }
	}
}

macro_rules! impl_display_signed {
    ($($t:ty),*) => {
        $(
            impl Display for $t {
                fn format(&self, f: &mut Formatter) -> Result<(), Error> {
                    let mut buf = [0u8; 64];
                    let len = i128_to_str((*self).into(), &mut buf, 10);
                    unsafe { f.write_str(from_utf8_unchecked(&buf), len) }
                }
            }
        )*
    };
}

impl_display_signed!(i8, i16, i32, i64, i128);

impl Display for f64 {
	fn format(&self, f: &mut Formatter) -> Result<(), Error> {
		let mut buf = [0u8; 512];
		let len = safe_f64_to_str(*self, buf.as_mut_ptr(), 512);
		if len > 0 {
			unsafe { f.write_str(from_utf8_unchecked(&buf), len as usize) }
		} else {
			Err(ErrorKind::IO.into())
		}
	}
}

impl Display for bool {
	fn format(&self, f: &mut Formatter) -> Result<(), Error> {
		if *self {
			f.write_str("true", 4)
		} else {
			f.write_str("false", 5)
		}
	}
}

impl Display for f32 {
	fn format(&self, f: &mut Formatter) -> Result<(), Error> {
		let mut buf = [0u8; 512];
		let len = safe_f64_to_str((*self).into(), buf.as_mut_ptr(), 512);
		if len > 0 {
			unsafe { f.write_str(from_utf8_unchecked(&buf), len as usize) }
		} else {
			Err(ErrorKind::IO.into())
		}
	}
}

impl Display for &str {
	fn format(&self, f: &mut Formatter) -> Result<(), Error> {
		f.write_str(self, self.len())
	}
}
