use core::intrinsics::{unchecked_div, unchecked_rem};
use core::mem::size_of;
use err;
use std::blob::Blob;
use std::error::Error;
use std::error::ErrorKind::{Alloc, CapacityExceeded, IllegalArgument};
use std::result::{Result, Result::Err, Result::Ok};
use std::util::copy_slice;
use sys::{ctzl, map};

macro_rules! bits_len {
	() => {{
		(page_size!() / 8)
	}};
}

pub struct BitMap {
	blob: Blob,
	page_count: usize,
	last_index: usize,
}

impl BitMap {
	pub fn new(pages: usize) -> Result<Self, Error> {
		if pages == 0 {
			Err(err!(IllegalArgument))
		} else {
			match Blob::new(pages) {
				Ok(blob) => Ok(BitMap {
					blob,
					page_count: 0,
					last_index: 0,
				}),
				Err(e) => Err(e),
			}
		}
	}

	pub fn extend(&mut self) -> Result<(), Error> {
		let pages = self.blob.pages();
		if (self.page_count + 1) >= bits_len!() * self.blob.pages() {
			return Err(err!(CapacityExceeded));
		}
		let next_page = unsafe { map(1) };
		if next_page.is_null() {
			return Err(err!(Alloc));
		}

		let len = pages * page_size!();
		let blob = self.blob.get_mut(0, len).unwrap();

		let ptr = blob.as_ptr().wrapping_add(self.page_count) as *mut u64;
		unsafe {
			*ptr = next_page as u64;
		}

		self.page_count += 1;

		Ok(())
	}

	pub fn allocate(&mut self) -> Result<usize, Error> {
		let pages = self.blob.pages();
		let bits_len = bits_len!();
		let page_size = page_size!();
		let mut index = self.last_index;
		let len = pages * page_size;
		let blob = self.blob.get_mut(0, len).unwrap();
		let v = unsafe { unchecked_div(index, bits_len) };
		let ptr = blob.as_ptr().wrapping_add(v * size_of::<u64>());
		let mut cur = ptr as *mut u64;

		while unsafe { *cur != 0 } {
			let mut ptr = unsafe { *cur } as *mut u64;
			let v = unsafe { unchecked_rem(index, bits_len) };
			ptr = ptr.wrapping_add(v);

			let mut found: bool;
			let mut x = 0;

			loop {
				let initial = aload!(ptr);
				found = initial != 0xFFFFFFFFFFFFFFFF;
				if !found {
					break;
				}
				x = unsafe { ctzl(!initial) };
				let updated = initial | (0x1u64 << x);
				if cas!(ptr, &initial, updated) {
					break;
				}
			}

			if found {
				self.last_index = index;
				return Ok(index << 6 | x as usize);
			}

			index += 1;
			let v = unsafe { unchecked_rem(index, bits_len) };
			if v == 0 {
				cur = cur.wrapping_add(1);
			}
			if index == pages * page_size / 8 {
				break;
			}
		}

		Err(err!(CapacityExceeded))
	}

	pub fn free(&mut self, _id: usize) {}

	fn _resize(&mut self, pages: usize) -> Result<(), Error> {
		match Blob::new(pages) {
			Ok(blob) => {
				let cur_pages = self.blob.pages();
				let copy_pages = if cur_pages > pages { pages } else { cur_pages };
				let len = copy_pages * page_size!();
				// unwrap ok because len <= pages * page_size!()
				let b0 = blob.get_mut(0, len).unwrap();
				let b1 = self.blob.get(0, len).unwrap();
				copy_slice(b1, b0, len);
				self.blob = blob;
				Ok(())
			}
			Err(e) => Err(e),
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;
	#[test]
	fn test_bitmap() {
		let mut b1 = BitMap::new(1).unwrap();
		assert!(b1.allocate().is_err());
		assert!(b1.extend().is_ok());
		for i in 0..16384 * 8 {
			assert_eq!(b1.allocate().unwrap(), i);
		}
		assert!(b1.allocate().is_err());
	}
}
