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
		if index < self.lrus.len() {
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
		if index < self.lrus.len() {
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

#[cfg(test)]
mod test {
	use super::*;
	#[test]
	fn test_dkm1() {
		let initial = crate::sys::safe_getalloccount();
		{
			let fs_name = ".test_dkm1";
			crate::sys::safe_init_fs(fs_name);
			let config = LruConfig {
				capacity: 2,
				..Default::default()
			};
			let mut dkm = DiskCacheManager::new(4, config).unwrap();

			let mut block1 = dkm.get_block(0, 1).unwrap();
			block1[0] = 100;
			assert!(dkm.release_block(block1).is_ok());
			let block1 = dkm.get_block(0, 2).unwrap();
			assert_eq!(block1[0], 100);

			let mut block2 = dkm.get_block(1, 1).unwrap();
			block2[0] = 101;
			assert!(dkm.release_block(block2).is_ok());

			let mut block3 = dkm.get_block(2, 1).unwrap();
			block3[0] = 102;
			assert!(dkm.release_block(block3).is_ok());

			let mut block4 = dkm.get_block(3, 1).unwrap();
			block4[0] = 103;
			assert!(dkm.release_block(block4).is_ok());

			let block1 = dkm.get_block(0, 1).unwrap();
			assert_eq!(block1[0], 100);
			assert!(dkm.release_block(block1).is_ok());

			let cur = crate::sys::safe_getalloccount();
			let block2 = dkm.get_block(1, 1).unwrap();
			// we're at capacity so a block should be freed that corresponds to this
			assert_eq!(cur, crate::sys::safe_getalloccount());

			assert_eq!(block2[0], 101);
			assert!(dkm.release_block(block2).is_ok());

			let block3 = dkm.get_block(2, 1).unwrap();
			assert_eq!(block3[0], 102);
			assert!(dkm.release_block(block3).is_ok());

			let block4 = dkm.get_block(3, 1).unwrap();
			assert_eq!(block4[0], 103);
			assert!(dkm.release_block(block4).is_ok());

			crate::sys::shutdown_fs(fs_name);
		}
		assert_eq!(initial, crate::sys::safe_getalloccount());
	}
}
