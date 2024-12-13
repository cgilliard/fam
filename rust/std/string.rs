use core::ops::Drop;
use core::ptr::{copy_nonoverlapping, null_mut};
use core::slice::from_raw_parts;
use core::str::from_utf8_unchecked;
use err;
use std::clone::Clone;
use std::error::{Error, ErrorKind::Alloc, ErrorKind::OutOfBounds};
use std::option::{Option, Option::None, Option::Some};
use std::result::{Result, Result::Err, Result::Ok};
use std::util::strcmp;
use sys::{map, unmap};

pub struct String {
	ptr: *mut u8,
	len: usize,
	sso: [u8; 32],
}

impl Clone for String {
	fn clone(&self) -> Result<Self, Error> {
		let ptr;
		let len = self.len;
		let mut sso = [0u8; 32];
		if len > 32 {
			let pages = pages!(len);
			unsafe {
				ptr = map(pages);
				if ptr.is_null() {
					return Err(err!(Alloc));
				}
				copy_nonoverlapping(self.ptr, ptr, len);
			}
		} else {
			ptr = null_mut();
			unsafe {
				copy_nonoverlapping(self.sso.as_ptr(), sso.as_mut_ptr(), len);
			}
		}

		Ok(Self { ptr, len, sso })
	}
}

impl String {
	pub fn new(s: &str) -> Result<Self, Error> {
		let len = s.len();
		let mut sso = [0u8; 32];
		let ptr;
		if len > 32 {
			let pages = pages!(len);
			ptr = unsafe { map(pages) };
			if ptr.is_null() {
				return Err(err!(Alloc));
			}
			unsafe {
				copy_nonoverlapping(s.as_ptr(), ptr, len);
			}
		} else {
			unsafe {
				copy_nonoverlapping(s.as_ptr(), sso.as_mut_ptr(), len);
			}
			ptr = null_mut()
		}

		Ok(Self { ptr, len, sso })
	}

	pub fn empty() -> Self {
		Self {
			ptr: null_mut(),
			len: 0,
			sso: [0u8; 32],
		}
	}

	pub fn to_str(&self) -> &str {
		if self.len > 32 {
			unsafe { from_utf8_unchecked(from_raw_parts(self.ptr, self.len as usize)) }
		} else {
			unsafe {
				let ptr = self.sso.as_ptr();
				let slice = from_raw_parts(ptr, self.len as usize);
				from_utf8_unchecked(slice)
			}
		}
	}

	pub fn find(&self, s: &str) -> Option<usize> {
		let mut x = self.to_str().as_ptr();
		let mut len = self.len() as usize;
		let s_len = s.len();

		if s_len == 0 {
			return Some(0);
		}

		unsafe {
			while len >= s_len {
				let v = from_utf8_unchecked(from_raw_parts(x, s_len));
				if strcmp(v, s) == 0 {
					return Some(self.len() as usize - len);
				}
				len -= 1;
				x = x.wrapping_add(1);
			}
		}
		None
	}

	pub fn rfind(&self, s: &str) -> Option<usize> {
		let s_len = s.len();
		let str_len = self.len() as usize;

		if s_len == 0 {
			return Some(str_len);
		}
		if s_len > str_len {
			return None;
		}

		let mut x = self.to_str().as_ptr().wrapping_add(str_len - s_len);
		let mut len = str_len;

		unsafe {
			while len >= s_len {
				let v = from_utf8_unchecked(from_raw_parts(x, s_len));
				if strcmp(v, s) == 0 {
					return Some(x as usize - self.to_str().as_ptr() as usize);
				}
				len -= 1;
				x = x.wrapping_sub(1);
			}
		}
		None
	}

	pub fn substring(&self, start: usize, end: usize) -> Result<&str, Error> {
		if end > self.len || start > end {
			return Err(err!(OutOfBounds));
		}
		let x = self.ptr.wrapping_add(start);
		Ok(unsafe { from_utf8_unchecked(from_raw_parts(x, end - start)) })
	}

	pub fn len(&self) -> usize {
		self.len
	}
}

impl Drop for String {
	fn drop(&mut self) {
		if self.len > 32 {
			let pages = pages!(self.len);
			unsafe {
				unmap(self.ptr, pages);
			}
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;
	#[test]
	fn test_strings() {
		let x1 = String::new("abc").unwrap();
		assert_eq!(x1.len(), 3);
		assert_eq!(x1.to_str(), "abc");
		assert_eq!(x1.find("bc"), Some(1));

		assert_eq!(x1.find("aa"), None);
		assert_eq!(x1.find(""), Some(0));
		let x2 = String::new("").unwrap();
		assert_eq!(x2.len(), 0);
		let x3 = String::new("aaabbbcccaaa").unwrap();
		assert_eq!(x3.rfind("aaa"), Some(9));
		assert_eq!(x3.rfind("ajlsfdjklasdjlfalsjkdfjklasdf"), None);
		assert_eq!(x3.rfind("aaaa"), None);
		assert_eq!(x3.find("ajlsfdjklasdjlfalsjkdfjklasdf"), None);
		let x4 = String::new("0123456789012345678901234567890123456789").unwrap();
		assert_eq!(x4.find("012"), Some(0));

		let x5 = x4.clone().unwrap();
		assert_eq!(x5.find("012"), Some(0));
		assert_eq!(x5.rfind("012"), Some(30));

		let x6 = x5.substring(5, 15).unwrap();
		let x7 = x6.as_bytes();
		assert_eq!(x7.len(), 10);
		assert_eq!(x7[0], b'5');
		let x8 = x5.substring(6, 6).unwrap();
		assert_eq!(x8.len(), 0);

		let x9 = match String::new("test") {
			Ok(s) => s,
			Err(_e) => String::empty(),
		};
		assert_eq!(x9.len(), 4);
	}
}
