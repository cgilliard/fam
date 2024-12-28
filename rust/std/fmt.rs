use core::ptr::copy_nonoverlapping;
use core::str::from_utf8_unchecked;
use prelude::*;
use sys::safe_f64_to_str;

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
