use core::ops::Drop;
use core::ptr::{copy_nonoverlapping, null_mut};
use std::clone::Clone;
use std::error::{Error, ErrorKind::Alloc};
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
		let ptr = if len > 32 {
			let pages = pages!(len - 32);
			let ptr = unsafe { map(pages) };
			if ptr.is_null() {
				return Err(err!(Alloc));
			}
			ptr
		} else {
			null_mut()
		};

		let mut sso = [0u8; 32];
		let bytes = s.as_bytes();
		let min = if len > 32 { 32 } else { len } as usize;
		unsafe {
			copy_nonoverlapping(bytes.as_ptr(), sso.as_mut_ptr(), min);
		}

		Ok(Self { ptr, len, sso })
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
