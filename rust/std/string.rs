use core::ops::Drop;
use core::ptr::{copy_nonoverlapping, null_mut};
use core::slice::from_raw_parts;
use core::str::from_utf8_unchecked;
use err;
use std::clone::Clone;
use std::error::{Error, ErrorKind::Alloc};
use std::option::{Option, Option::None, Option::Some};
use std::result::{Result, Result::Err, Result::Ok};
use sys::{cstring_len, map, unmap};

pub struct String {
	ptr: *mut u8,
	len: usize,
	sso: [u8; 32],
}

impl String {
	pub fn new(s: &str) -> Result<Self, Error> {
		/*
		//	let len = unsafe { cstring_len(s) } as usize;
		let mut sso = [0u8; 32];
		let ptr: *mut u8;
		if len > 32 {
			let pages = pages!(len);
			ptr = unsafe { map(pages) };
			if ptr.is_null() {
				return Err(err!(Alloc));
			}
			unsafe {
				copy_nonoverlapping(s, ptr, len);
			}
		} else {
			unsafe {
				copy_nonoverlapping(s, sso.as_mut_ptr(), len);
			}
			ptr = null_mut()
		}

		//		let x = "".len();
				*/
		let len = s.len();
		let ptr = null_mut();
		let sso = [0u8; 32];

		Ok(Self { ptr, len, sso })
	}

	pub fn to_ptr(&self) -> *const u8 {
		if self.len > 32 {
			self.ptr
		} else {
			self.sso.as_ptr()
		}
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
