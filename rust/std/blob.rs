use core::ops::Drop;
use core::slice::{from_raw_parts, from_raw_parts_mut};
use err;
use std::error::{Error, ErrorKind::Alloc};
use std::result::{Result, Result::Err, Result::Ok};
use sys::{map, unmap};

pub struct Blob {
	ptr: *mut u8,
	pages: usize,
}

impl Drop for Blob {
	fn drop(&mut self) {
		unsafe {
			unmap(self.ptr, self.pages);
		}
	}
}

impl Blob {
	pub fn new(pages: usize) -> Result<Self, Error> {
		let ptr = unsafe { map(pages) };
		if ptr.is_null() {
			Err(err!(Alloc))
		} else {
			Ok(Self { ptr, pages })
		}
	}

	pub fn get(&self) -> &[u8] {
		let byte_size = self.pages * page_size!();
		unsafe { from_raw_parts(self.ptr, byte_size) }
	}

	pub fn get_mut(&mut self) -> &mut [u8] {
		let byte_size = self.pages * page_size!();
		unsafe { from_raw_parts_mut(self.ptr, byte_size) }
	}
}

#[cfg(test)]
mod test {
	use super::*;
	#[test]
	fn test_blob() {
		let mut b1 = Blob::new(1).unwrap();
		let a1 = b1.get_mut();
		for i in 0..1000 {
			a1[i] = (i % 26) as u8 + b'a';
		}
		let a2 = b1.get();
		for i in 0..1000 {
			assert_eq!(a2[i], (i % 26) as u8 + b'a');
		}
	}
}
