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
	pub fn get_block(&mut self, page_offset: u64, page_count: usize) -> Result<Block, Error> {
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

				// SAFETY: block.clone does not fail
				let ret = (*block).clone().unwrap();
				match lru.insert(block) {
					Some(rem) => {
						unsafe {
							drop_in_place(rem.raw());
						}
						rem.release();
					}
					None => {}
				}
				Ok(ret)
			} else {
				Ok((*ptr).clone().unwrap())
			}
		} else {
			Err(err!(OutOfBounds))
		}
	}
	pub fn release_block(&mut self, block: Block) -> Result<(), Error> {
		let index = block.pages().trailing_zeros() as usize;
		if index >= self.lrus.len() {
			let lru = &mut self.lrus[index];
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
