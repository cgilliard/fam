extern crate core;
use core::cmp::PartialEq;
use core::convert::From;
use core::fmt::Debug;
use core::fmt::Formatter;
use core::ptr::copy_nonoverlapping;
use core::slice::from_raw_parts;
use core::str::from_utf8_unchecked;
use prelude::*;
use std::util::strcmp;

pub struct String {
	value: Rc<Box<[u8]>>,
	end: usize,
	start: usize,
}

impl Debug for String {
	fn fmt(&self, _f: &mut Formatter<'_>) -> core::result::Result<(), core::fmt::Error> {
		// TODO: we don't write to the formatter because we don't seem to have write! in
		// rustc or mrustc
		// Just implementing this so assert_eq works
		core::result::Result::Ok(())
	}
}

impl PartialEq for String {
	fn eq(&self, other: &String) -> bool {
		strcmp(self.to_str(), other.to_str()) == 0
	}
}

impl Clone for String {
	fn clone(&self) -> Result<Self, Error> {
		match self.value.clone() {
			Ok(value) => Ok(Self {
				value,
				start: self.start,
				end: self.end,
			}),
			Err(e) => Err(e),
		}
	}
}

impl From<&str> for String {
	fn from(s: &str) -> Self {
		Self::new(s).unwrap()
	}
}

impl String {
	pub fn new(s: &str) -> Result<Self, Error> {
		let end = s.len();
		let start = 0;
		match Box::new_zeroed_byte_slice(end) {
			Ok(mut value) => {
				let valueptr = value.as_mut_ptr() as *mut u8;
				unsafe {
					copy_nonoverlapping(s.as_ptr(), valueptr, end);
				}
				Ok(Self {
					value: Rc::new(value).unwrap(),
					start,
					end,
				})
			}
			Err(e) => Err(e),
		}
	}

	pub fn to_str(&self) -> &str {
		let ptr = self.value.get().as_ptr() as *const u8;
		let ptr = unsafe { ptr.add(self.start) };
		unsafe { from_utf8_unchecked(from_raw_parts(ptr, self.end - self.start)) }
	}

	pub fn substring(&self, start: usize, end: usize) -> Result<Self, Error> {
		if start > end || end - start > self.len() {
			Err(ErrorKind::OutOfBounds.into())
		} else {
			match self.value.clone() {
				Ok(value) => Ok(Self {
					value,
					start: start + self.start,
					end: self.start + end,
				}),
				Err(e) => Err(e),
			}
		}
	}

	pub fn len(&self) -> usize {
		self.end - self.start
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
}

#[cfg(test)]
mod test {
	use super::*;
	use core::convert::Into;
	use std::boxed::assert_all_slabs_free;
	use sys::getalloccount;

	#[test]
	fn test_strings() {
		let initial = unsafe { getalloccount() };
		{
			{
				let x1 = String::new("abcdefghijkl").unwrap();
				assert_eq!(x1.len(), 12);
				assert_eq!(x1.to_str(), "abcdefghijkl");
				assert_eq!(x1.substring(3, 6).unwrap().to_str(), "def");
				let x2 = x1.substring(3, 9).unwrap();
				assert_eq!(x2.to_str(), "defghi");
				assert_eq!(x2, String::new("defghi").unwrap());
				assert_eq!(x2, "defghi".into());
				assert_eq!(x1, String::new("abcdefghijkl").unwrap());

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
				let x7 = x6.to_str().as_bytes();
				assert_eq!(x7.len(), 10);
				assert_eq!(x7[0], b'5');
				let x8 = x5.substring(6, 6).unwrap();
				assert_eq!(x8.len(), 0);

				let x9 = match String::new("test") {
					Ok(s) => s,
					Err(_e) => String::new("").unwrap(),
				};
				assert_eq!(x9.len(), 4);
			}

			assert_all_slabs_free();
			unsafe {
				cleanup_slab_allocators();
			}
		}
		assert_eq!(initial, unsafe { getalloccount() });
	}
}
