use core::ptr::copy_nonoverlapping;
use core::str::from_utf8_unchecked;
use prelude::*;
use std::util::i128_to_str;
use std::util::u128_to_str;
use sys::f64_to_str;

#[macro_export]
macro_rules! write {
    ($f:expr, $fmt:expr, $($t:expr),*) => {{
        #[allow(unused_mut)]
        {
            let mut err: Error = ErrorKind::Unknown.into();
            $(
                if err.kind == ErrorKind::Unknown {
                    match $t.fmt(&mut $f) {
                        Ok(_) => {},
                        Err(e) => err = e,
                    }
                }
            )*

            if err.kind != ErrorKind::Unknown {
                Err(err)
            } else {
                Ok(())
            }
        }
    }};
}

pub struct Formatter {
	buffer: Vec<u8>,
	pos: usize,
}

pub trait Display {
	fn fmt(&self, f: &mut Formatter) -> Result<(), Error>;
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
		unsafe { from_utf8_unchecked(&self.buffer[0..self.pos]) }
	}
}

macro_rules! impl_display_unsigned {
    ($($t:ty),*) => {
        $(
            impl Display for $t {
                fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
                    let mut buf = [0u8; 64];
                    let len = u128_to_str((*self).into(), 0, &mut buf);
                    unsafe { f.write_str(from_utf8_unchecked(&buf), len) }
                }
            }
        )*
    };
}

impl_display_unsigned!(u8, u16, u32, u64, u128);

macro_rules! impl_display_signed {
    ($($t:ty),*) => {
        $(
            impl Display for $t {
                fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
                    let mut buf = [0u8; 64];
                    let len = i128_to_str((*self).into(), &mut buf);
                    unsafe { f.write_str(from_utf8_unchecked(&buf), len) }
                }
            }
        )*
    };
}

impl_display_signed!(i8, i16, i32, i64, i128);

impl Display for f64 {
	fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
		let mut buf = [0u8; 512];
		let len = unsafe { f64_to_str(*self, buf.as_mut_ptr(), 512) };
		if len > 0 {
			unsafe { f.write_str(from_utf8_unchecked(&buf), len as usize) }
		} else {
			Err(ErrorKind::IO.into())
		}
	}
}

impl Display for f32 {
	fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
		let mut buf = [0u8; 512];
		let len = unsafe { f64_to_str((*self).into(), buf.as_mut_ptr(), 512) };
		if len > 0 {
			unsafe { f.write_str(from_utf8_unchecked(&buf), len as usize) }
		} else {
			Err(ErrorKind::IO.into())
		}
	}
}

impl Display for &str {
	fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
		f.write_str(self, self.len())
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn test_formatter1() {
		let mut fmt = Formatter::new();
		fmt.write_str("ok1", 3).unwrap();
		fmt.write_str("hi hi hi", 8).unwrap();
		fmt.write_str(" ", 1).unwrap();
		fmt.write_str("7", 1).unwrap();
		assert_eq!(fmt.as_str(), "ok1hi hi hi 7");
		let mut fmt = Formatter::new();
		fmt.write_str("===", 3).unwrap();
		166u64.fmt(&mut fmt).unwrap();
		fmt.write_str("===", 3).unwrap();
		assert_eq!(fmt.as_str(), "===166===");
	}

	#[test]
	fn test_formatter_unsigned() {
		let mut fmt = Formatter::new();
		1234u128.fmt(&mut fmt).unwrap();
		fmt.write_str("===", 3).unwrap();
		1234u64.fmt(&mut fmt).unwrap();
		fmt.write_str("===", 3).unwrap();
		1234u32.fmt(&mut fmt).unwrap();
		fmt.write_str("===", 3).unwrap();
		1234u16.fmt(&mut fmt).unwrap();
		fmt.write_str("===", 3).unwrap();
		123u8.fmt(&mut fmt).unwrap();
		assert_eq!(fmt.as_str(), "1234===1234===1234===1234===123");
	}

	#[test]
	fn test_formatter_signed() {
		let mut fmt = Formatter::new();
		(-1234i128).fmt(&mut fmt).unwrap();
		fmt.write_str("===", 3).unwrap();
		(-1234i64).fmt(&mut fmt).unwrap();
		fmt.write_str("===", 3).unwrap();
		(-1234i32).fmt(&mut fmt).unwrap();
		fmt.write_str("===", 3).unwrap();
		(-1234i16).fmt(&mut fmt).unwrap();
		fmt.write_str("===", 3).unwrap();
		(-123i8).fmt(&mut fmt).unwrap();
		assert_eq!(fmt.as_str(), "-1234===-1234===-1234===-1234===-123");

		let mut fmt = Formatter::new();
		1234i128.fmt(&mut fmt).unwrap();
		fmt.write_str("===", 3).unwrap();
		1234i64.fmt(&mut fmt).unwrap();
		fmt.write_str("===", 3).unwrap();
		1234i32.fmt(&mut fmt).unwrap();
		fmt.write_str("===", 3).unwrap();
		1234i16.fmt(&mut fmt).unwrap();
		fmt.write_str("===", 3).unwrap();
		123i8.fmt(&mut fmt).unwrap();
		assert_eq!(fmt.as_str(), "1234===1234===1234===1234===123");
	}

	#[test]
	fn test_float() {
		let mut fmt = Formatter::new();
		(-123.456f64).fmt(&mut fmt).unwrap();
		fmt.write_str("===", 3).unwrap();
		(123.1f32).fmt(&mut fmt).unwrap();
		assert_eq!(fmt.as_str(), "-123.45600===123.10000");
	}

	#[test]
	fn test_fmt() {
		let mut f = Formatter::new();
		assert!(write!(f, "test", 1u8, " ", -23i128, "this is a test").is_ok());
		assert_eq!(f.as_str(), "1 -23this is a test");
	}
}
