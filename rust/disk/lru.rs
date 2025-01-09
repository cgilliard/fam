#![allow(dead_code)]
#![allow(unused_variables)]

use prelude::*;
use sys::{safe_fmap, safe_getpagesize, safe_unmap};

pub struct LruConfig {
	arr_size: usize,
	capacity: usize,
}

pub struct Block {
	next: Ptr<Block>,
	prev: Ptr<Block>,
	chain_next: Ptr<Block>,
	id: u64,
	pages: usize,
	data: Vec<u8>,
}

pub struct Lru {
	arr: Vec<Ptr<Block>>,
	head: Ptr<Block>,
	tail: Ptr<Block>,
	count: usize,
	capacity: usize,
}

impl Drop for Block {
	fn drop(&mut self) {
		safe_unmap(self.data.as_ptr(), self.pages);
	}
}

impl Block {
	pub fn new(id: u64, pages: usize) -> Result<Self, Error> {
		let page_size = safe_getpagesize();

		let data = safe_fmap(id as i64, pages);
		if data.is_null() {
			return Err(err!(Alloc));
		}

		let data = unsafe {
			let mut ret = Vec::from_raw_parts(data, pages * page_size);
			ret.leak();
			ret
		};

		Ok(Self {
			chain_next: Ptr::null(),
			next: Ptr::null(),
			prev: Ptr::null(),
			id,
			pages,
			data,
		})
	}

	pub fn from_raw(id: u64, pages: usize, data: Vec<u8>) -> Self {
		Self {
			next: Ptr::null(),
			prev: Ptr::null(),
			chain_next: Ptr::null(),
			id,
			pages,
			data,
		}
	}

	pub fn as_vec(&self) -> Vec<u8> {
		unsafe { Vec::from_raw_parts(self.data.as_ptr(), self.pages * safe_getpagesize()) }
	}

	pub fn as_mut_slice(&mut self) -> &mut [u8] {
		self.data.as_mut_slice()
	}
	pub fn as_slice(&self) -> &[u8] {
		self.data.as_slice()
	}
}

impl Default for LruConfig {
	fn default() -> Self {
		Self {
			arr_size: 1024,
			capacity: 128,
		}
	}
}

impl Lru {
	pub fn new(config: &LruConfig) -> Result<Self, Error> {
		let mut arr = Vec::new();
		match arr.resize(config.arr_size) {
			Ok(_) => {}
			Err(e) => return Err(e),
		}

		Ok(Self {
			arr,
			head: Ptr::null(),
			tail: Ptr::null(),
			count: 0,
			capacity: config.capacity,
		})
	}

	pub fn insert(&mut self, mut block: Ptr<Block>) -> Option<Ptr<Block>> {
		block.next = self.head;
		if !self.head.is_null() {
			self.head.prev = block;
		} else {
			self.tail = block;
		}
		self.head = block;
		block.prev = Ptr::null();
		block.chain_next = Ptr::null();
		let slot = murmur3_32_of_u64(block.id, get_murmur_seed()) as usize % self.arr.len();
		let mut cur = self.arr[slot];
		if cur.is_null() {
			self.arr[slot] = block;
		} else {
			while !cur.is_null() {
				if cur.chain_next.is_null() {
					cur.chain_next = block;
					break;
				}
				cur = cur.chain_next;
			}
		}
		self.count += 1;
		if self.count > self.capacity {
			let ret = self.tail;
			self.tail = self.tail.prev;
			self.tail.next = Ptr::null();
			let _ = self.remove(ret.id);
			Some(ret)
		} else {
			None
		}
	}

	pub fn find(&self, id: u64) -> Result<Ptr<Block>, Error> {
		let slot = murmur3_32_of_u64(id, get_murmur_seed()) as usize % self.arr.len();
		let mut cur = self.arr[slot];
		while !cur.is_null() && cur.id != id {
			cur = cur.chain_next;
		}
		Ok(cur)
	}

	pub fn remove(&mut self, id: u64) -> Ptr<Block> {
		let slot = murmur3_32_of_u64(id, get_murmur_seed()) as usize % self.arr.len();
		let mut cur = self.arr[slot];
		let mut last = cur;
		while !cur.is_null() && cur.id != id {
			last = cur;
			cur = cur.chain_next;
		}
		if !cur.is_null() {
			self.count -= 1;

			if cur.raw() == self.arr[slot].raw() {
				self.arr[slot] = self.arr[slot].chain_next;
			} else {
				last.chain_next = cur.chain_next;
			}
			if !cur.next.is_null() {
				cur.next.prev = cur.next;
			} else {
				self.tail = cur.prev;
			}
			if !cur.prev.is_null() {
				cur.prev.next = cur.prev;
			} else {
				self.head = cur.next;
			}
		}

		cur
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use core::ptr::drop_in_place;

	#[test]
	fn test_lru1() {
		let initial = crate::sys::safe_getalloccount();
		{
			let fs_name = ".test_lru1";
			crate::sys::safe_init_fs(fs_name);
			let mut lru = Lru::new(&LruConfig::default()).unwrap();
			let mut block = Block::new(0, 1).unwrap();
			block.as_mut_slice()[0] = 1;
			block.as_mut_slice()[1] = 2;
			let ptr = Ptr::alloc(block).unwrap();
			assert!(lru.insert(ptr).is_none());

			let f = lru.find(0).unwrap();
			assert_eq!((*f).as_slice()[0], 1);
			assert_eq!((*f).as_slice()[1], 2);
			let mut rem = lru.remove(0);
			assert_eq!((*rem).as_slice()[0], 1);
			assert_eq!((*rem).as_slice()[1], 2);
			(*rem).as_mut_slice()[0] = 0;
			(*rem).as_mut_slice()[1] = 0;
			assert_eq!((*rem).as_slice()[0], 0);
			assert_eq!((*rem).as_slice()[1], 0);
			unsafe {
				drop_in_place(rem.raw());
			}
			rem.release();
			crate::sys::shutdown_fs(fs_name);
		}
		assert_eq!(initial, crate::sys::safe_getalloccount());
	}

	#[test]
	fn test_lru_evict() {
		let initial = crate::sys::safe_getalloccount();
		{
			let fs_name = ".test_lru_evict";
			crate::sys::safe_init_fs(fs_name);
			let mut lru = Lru::new(&LruConfig {
				capacity: 4,
				arr_size: 1,
				..LruConfig::default()
			})
			.unwrap();
			let mut block = Block::new(0, 1).unwrap();
			block.as_mut_slice()[0] = 1;
			let ptr = Ptr::alloc(block).unwrap();
			assert!(lru.insert(ptr).is_none());

			let mut block = Block::new(1, 1).unwrap();
			block.as_mut_slice()[0] = 2;
			let ptr = Ptr::alloc(block).unwrap();
			assert!(lru.insert(ptr).is_none());

			let mut block = Block::new(2, 1).unwrap();
			block.as_mut_slice()[0] = 3;
			let ptr = Ptr::alloc(block).unwrap();
			assert!(lru.insert(ptr).is_none());

			let mut block = Block::new(3, 1).unwrap();
			block.as_mut_slice()[0] = 4;
			let ptr = Ptr::alloc(block).unwrap();
			assert!(lru.insert(ptr).is_none());

			let mut block = Block::new(4, 1).unwrap();
			block.as_mut_slice()[0] = 5;
			let ptr = Ptr::alloc(block).unwrap();
			let rem = lru.insert(ptr).unwrap();
			assert_eq!((*rem).as_slice()[0], 1);
			unsafe {
				drop_in_place(rem.raw());
			}
			rem.release();

			let rem = lru.remove(1);

			unsafe {
				drop_in_place(rem.raw());
			}
			rem.release();
			let rem = lru.remove(2);
			unsafe {
				drop_in_place(rem.raw());
			}
			rem.release();

			let rem = lru.remove(3);
			unsafe {
				drop_in_place(rem.raw());
			}
			rem.release();
			let rem = lru.remove(4);
			unsafe {
				drop_in_place(rem.raw());
			}
			rem.release();

			crate::sys::shutdown_fs(fs_name);
		}
		assert_eq!(initial, crate::sys::safe_getalloccount());
	}
}
