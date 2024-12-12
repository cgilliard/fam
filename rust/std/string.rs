use core::ops::Drop;
use core::ptr::{copy_nonoverlapping, null_mut};
use core::slice::from_raw_parts;
use core::str::from_utf8_unchecked;
use std::clone::Clone;
use std::error::{Error, ErrorKind::Alloc};
use std::option::{Option, Option::None, Option::Some};
use std::result::{Result, Result::Err, Result::Ok};
use sys::{map, unmap};

pub struct String {
	ptr: *mut u8,
	len: u64,
	sso: [u8; 32],
}

impl Clone for String {
	fn clone(&self) -> Result<Self, Error> {
		let ptr: *mut u8 = if !self.ptr.is_null() {
			let pages = pages!(self.len - 32);
			unsafe { map(pages) }
		} else {
			null_mut()
		};

		if ptr.is_null() && !self.ptr.is_null() {
			Err(err!(Alloc))
		} else {
			if !ptr.is_null() {
				unsafe {
					copy_nonoverlapping(self.ptr, ptr, (self.len - 32) as usize);
				}
			}
			Ok(Self {
				ptr,
				len: self.len,
				sso: self.sso,
			})
		}
	}
}

impl String {
	pub fn new(s: &str) -> Result<Self, Error> {
		let len = s.len() as u64;
		let mut sso = [0u8; 32];
		let ptr;
		if len > 32 {
			let pages = pages!(len);
			ptr = unsafe { map(pages) };
			if ptr.is_null() {
				return Err(err!(Alloc));
			}
			let bytes = s.as_bytes();
			unsafe {
				copy_nonoverlapping(bytes.as_ptr(), ptr, bytes.len());
			}
		} else {
			let bytes = s.as_bytes();
			unsafe {
				copy_nonoverlapping(bytes.as_ptr(), sso.as_mut_ptr(), bytes.len());
			}
			ptr = null_mut()
		}

		Ok(Self { ptr, len, sso })
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

	pub fn len(&self) -> u64 {
		self.len
	}

	pub fn cmp(a: &str, b: &str) -> i32 {
		let len = if a.len() > b.len() { b.len() } else { a.len() };
		let x = a.as_bytes();
		let y = b.as_bytes();
		let mut i = 0;
		while i < len {
			if x[i] != y[i] {
				break;
			}
			i += 1;
		}
		if i == len {
			0
		} else if i < len && x[i] > y[i] {
			1
		} else {
			-1
		}
	}

	pub fn find(&self, s: &str) -> Option<usize> {
		let mut x = self.to_str().as_ptr();
		let mut len = self.len() as usize;
		let mut i: usize = 0;
		while len >= s.len() {
			unsafe {
				let v = from_utf8_unchecked(from_raw_parts(x, len - i as usize));
				if Self::cmp(v, s) == 0 {
					return Some(self.len() as usize - len);
				}
			}
			len -= 1;
			x = x.wrapping_add(1);
			i += 1;
		}
		None
	}

	pub fn rfind(&self, s: &str) -> Option<usize> {
		let slen = s.len();
		let mut x = self.to_str().as_ptr();
		let mut len = self.len() as usize;
		if slen > len {
			return None;
		}
		x = x.wrapping_add(len - slen);
		let mut i = 0;
		while len >= slen {
			unsafe {
				let v = from_utf8_unchecked(from_raw_parts(x, (slen + i) as usize));
				if Self::cmp(v, s) == 0 {
					return Some((self.len() as usize - slen) - i);
				}
			}
			len -= 1;
			i += 1;
			x = x.wrapping_sub(1);
		}
		None
	}
}

impl Drop for String {
	fn drop(&mut self) {
		if self.len > 32 {
			let pages = pages!(self.len - 32);
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
	}
}
