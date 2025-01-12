use core::ptr::drop_in_place;
use disk::lru::{Block, Lru, LruConfig};
use prelude::*;

pub struct DiskCacheManager {
	lrus: Vec<Lru>,
}

impl DiskCacheManager {
	pub fn new(mut count: usize, config: LruConfig) -> Result<Self, Error> {
		let mut lrus = Vec::new();
		while count > 0 {
			count -= 1;
			let lru = match Lru::new(&config) {
				Ok(lru) => lru,
				Err(e) => return Err(e),
			};
			match lrus.push(lru) {
				Ok(_) => {}
				Err(e) => return Err(e),
			}
		}
		Ok(Self { lrus })
	}
	pub fn get_pages(&mut self, page_offset: u64, page_count: usize) -> Result<Vec<u8>, Error> {
		if page_count == 0 || (page_count & (page_count - 1)) != 0 {
			return Err(err!(IllegalArgument));
		}
		let index = page_count.trailing_zeros() as usize;
		if index >= self.lrus.len() {
			let lru = &mut self.lrus[index];
			let ptr = lru.remove(page_offset);
			if ptr.is_null() {
				let block = match Block::new(page_offset, page_count) {
					Ok(block) => block,
					Err(e) => return Err(e),
				};
				let block = match Ptr::alloc(block) {
					Ok(block) => block,
					Err(e) => return Err(e),
				};
				let ret = Block::from_raw(page_offset, page_count, block.as_vec());
				match lru.insert(block) {
					Some(rem) => {
						unsafe {
							drop_in_place(rem.raw());
						}
						rem.release();
					}
					None => {}
				}
				Ok(ret.as_vec())
			} else {
				Ok((*ptr).as_vec())
			}
		} else {
			Err(err!(OutOfBounds))
		}
	}
	pub fn release_pages(
		&mut self,
		page_offset: u64,
		page_count: usize,
		data: Vec<u8>,
	) -> Result<(), Error> {
		let index = page_count.trailing_zeros() as usize;
		if index >= self.lrus.len() {
			let lru = &mut self.lrus[index];
			let block = Block::from_raw(page_offset, page_count, data);
			let ptr = match Ptr::alloc(block) {
				Ok(ptr) => ptr,
				Err(e) => return Err(e),
			};
			match lru.insert(ptr) {
				Some(rem) => {
					unsafe {
						drop_in_place(rem.raw());
					}
					rem.release();
				}
				None => {}
			}
			Ok(())
		} else {
			Err(err!(OutOfBounds))
		}
	}
}
