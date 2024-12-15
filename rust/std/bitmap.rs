use core::mem::size_of;
use err;
use std::blob::Blob;
use std::error::Error;
use std::error::ErrorKind::{Alloc, CapacityExceeded, IllegalArgument};
use std::result::{Result, Result::Err, Result::Ok};
use std::util::{divide_usize, rem_usize};
use sys::{ctzl, map};

macro_rules! bits_len {
	() => {{
		(page_size!() / size_of::<u64>())
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

		let blob = self.blob.get_mut(0, pages * page_size!()).unwrap();

		let ptr = blob
			.as_ptr()
			.wrapping_add(self.page_count * size_of::<u64>()) as *mut u64;
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
		let u64_size = size_of::<u64>();
		let mut index = aload!(&(self.last_index as u64)) as usize;
		let first = index;
		let blob = self.blob.get_mut(0, pages * page_size).unwrap();
		let ptr = blob
			.as_ptr()
			.wrapping_add(divide_usize(index, bits_len) * u64_size);
		let mut cur = ptr as *mut u64;

		while unsafe { *cur != 0 } {
			let mut ptr = unsafe { *cur } as *mut u64;
			let v = rem_usize(index, bits_len);
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
				astore!(&mut (self.last_index as u64), index as u64);
				return Ok(index << 6 | x as usize);
			}

			index += 1;
			let v = rem_usize(index, bits_len);
			if v == 0 {
				cur = cur.wrapping_add(1);
			}
			if index == self.page_count * page_size / 8 {
				index = 0;
			}
			if index == first {
				break;
			}
		}

		Err(err!(CapacityExceeded))
	}

	pub fn free(&mut self, mut id: usize) {
		let pages = self.blob.pages();
		let bits_len = bits_len!();
		let page_size = page_size!();
		let u64_size = size_of::<u64>();
		let x = 1 << (id & 0x3F);
		id >>= 6;
		let len = pages * page_size;
		let blob = self.blob.get_mut(0, len).unwrap();
		let ptr = blob
			.as_ptr()
			.wrapping_add(divide_usize(id, bits_len) * u64_size);
		let cur = ptr as *mut u64;

		if id < aload!(&(self.last_index as u64)) as usize {
			astore!(&mut (self.last_index as u64), id as u64);
		}

		let mut ptr = unsafe { *cur } as *mut u64;
		let v = rem_usize(id, bits_len);
		ptr = ptr.wrapping_add(v);
		loop {
			let initial = aload!(ptr);
			let updated = initial & !x;
			if updated == initial {
				panic!("Double free attempt!");
			}
			if cas!(ptr, &initial, updated) {
				break;
			}
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
		assert!(b1.extend().is_ok());
		assert_eq!(b1.allocate().unwrap(), 16384 * 8);

		let mut b2 = BitMap::new(1).unwrap();
		assert!(b2.extend().is_ok());
		for i in 0..100 {
			assert_eq!(b2.allocate().unwrap(), i);
		}

		b2.free(3);
		assert_eq!(b2.allocate().unwrap(), 3);
		assert_eq!(b2.allocate().unwrap(), 100);

		b2.free(49);
		b2.free(55);
		b2.free(77);
		assert_eq!(b2.allocate().unwrap(), 49);
		assert_eq!(b2.allocate().unwrap(), 55);
		assert_eq!(b2.allocate().unwrap(), 77);
		assert_eq!(b2.allocate().unwrap(), 101);
	}
}
