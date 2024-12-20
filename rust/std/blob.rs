use core::ops::Drop;
use core::ptr::null_mut;
use core::slice::{from_raw_parts, from_raw_parts_mut};
use prelude::*;
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
		if pages == 0 {
			Ok(Self {
				ptr: null_mut(),
				pages: 0,
			})
		} else {
			let ptr = unsafe { map(pages) };
			if ptr.is_null() {
				Err(ErrorKind::Alloc.into())
			} else {
				Ok(Self { ptr, pages })
			}
		}
	}

	pub fn get(&self, offset: usize, len: usize) -> Result<&[u8], Error> {
		let byte_size = self.pages * page_size!();
		if offset + len > byte_size {
			Err(ErrorKind::OutOfBounds.into())
		} else {
			let ptr = self.ptr.wrapping_add(offset);
			Ok(unsafe { from_raw_parts(ptr, len) })
		}
	}

	pub fn get_mut(&self, offset: usize, len: usize) -> Result<&mut [u8], Error> {
		let byte_size = self.pages * page_size!();
		if offset + len > byte_size {
			Err(ErrorKind::OutOfBounds.into())
		} else {
			let ptr = self.ptr.wrapping_add(offset);
			Ok(unsafe { from_raw_parts_mut(ptr, len) })
		}
	}

	pub fn pages(&self) -> usize {
		self.pages
	}
}

#[cfg(test)]
mod test {
	use super::*;
	#[test]
	fn test_blob() {
		let b1 = Blob::new(8).unwrap();
		let a1 = b1.get_mut(0, 20000).unwrap();
		for i in 0..20000 {
			a1[i] = (i % 26) as u8 + b'a';
		}
		let a2 = b1.get(0, 20000).unwrap();
		for i in 0..20000 {
			assert_eq!(a2[i], (i % 26) as u8 + b'a');
		}

		let b2 = Blob::new(8).unwrap();
		let a3 = b2.get_mut(0, 20000).unwrap();
		a3[..20000].copy_from_slice(&a2[..20000]);
		for i in 0..20000 {
			assert_eq!(a3[i], (i % 26) as u8 + b'a');
		}
	}
}
