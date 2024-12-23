use core::ptr::copy_nonoverlapping;
use core::str::from_utf8_unchecked;
use prelude::*;

pub struct StringBuilder {
	buffer: Vec<u8>,
	pos: usize,
}

impl StringBuilder {
	pub fn new() -> Self {
		Self {
			buffer: Vec::new(),
			pos: 0,
		}
	}

	pub fn append(&mut self, s: &str) -> Result<(), Error> {
		let slen = s.len();
		match self.buffer.resize(self.pos + slen) {
			Ok(_) => {}
			Err(e) => return Err(e),
		}
		unsafe {
			let ptr = (self.buffer.as_mut_ptr() as *mut u8).add(self.pos);
			copy_nonoverlapping(s.as_ptr(), ptr, slen);
		}
		self.pos += slen;
		Ok(())
	}

	pub fn clear(&mut self) {
		self.pos = 0;
	}

	pub fn to_string(&self) -> Result<String, Error> {
		String::new(unsafe { from_utf8_unchecked(&self.buffer[0..self.pos]) })
	}
}

#[cfg(test)]
mod test {
	use super::*;
	#[test]
	fn test_string_builder() {
		let mut sb1 = StringBuilder::new();
		sb1.append("test123 ").unwrap();
		sb1.append("abc ").unwrap();
		sb1.append("1").unwrap();
		assert_eq!(
			sb1.to_string().unwrap(),
			String::new("test123 abc 1").unwrap()
		);
	}
}
