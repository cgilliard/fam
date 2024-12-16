use core::ops::Drop;
use core::ptr::null_mut;
use core::slice::{from_raw_parts, from_raw_parts_mut};
use err;
use std::bitmap::BitMap;
use std::error::{
	Error,
	ErrorKind::{Alloc, CapacityExceeded},
};
use std::lock::Lock;
use std::result::{Result, Result::Err, Result::Ok};
use std::util::{divide_usize, rem_usize};
use sys::{ctz, map, unmap};

macro_rules! shift_per_level {
	() => {{
		unsafe { ctz(crate::sys::getpagesize() as u32) - 3 }
	}};
}

macro_rules! offset {
	($z:expr, $slab_size:expr) => {{
		let entries = 0x1
			<< unsafe {
				ctz(divide_usize(crate::sys::getpagesize() as usize, $slab_size) as u32) as usize
			};
		rem_usize($z, entries) * $slab_size
	}};
}

macro_rules! mask {
	() => {{
		(0x1 << shift_per_level!()) - 1
	}};
}

pub struct SlabAllocator {
	data: *mut *mut *mut *mut u8,
	_head: *mut u8,
	_tail: *mut u8,
	bitmap: BitMap,
	slab_size: usize,
	_max_free_slabs: usize,
	_max_total_slabs: usize,
	lock: Lock,
	_free_slabs: usize,
	_total_slabs: usize,
}

#[derive(Copy, Clone)]
pub struct Slab {
	_next: *mut Slab,
	data: *mut u8,
	id: usize,
	len: usize,
}

impl Slab {
	pub fn get(&self) -> &[u8] {
		unsafe { from_raw_parts(self.data, self.len) }
	}

	pub fn get_mut(&mut self) -> &mut [u8] {
		unsafe { from_raw_parts_mut(self.data, self.len) }
	}
}

impl Drop for SlabAllocator {
	fn drop(&mut self) {
		unsafe {
			if !self.data.is_null() {
				let mut i = 0;
				while !(*self.data.add(i)).is_null() {
					let mut j = 0;
					while !(*(*self.data.add(i)).add(j)).is_null() {
						let mut k = 0;
						while !(*(*(*self.data.add(i)).add(j)).add(k)).is_null() {
							unmap(*(*(*self.data.add(i)).add(j)).add(k), 1);
							k += 1;
						}
						unmap(*(*self.data.add(i)).add(j) as *mut u8, 1);
						j += 1;
					}
					unmap(*self.data.add(i) as *mut u8, 1);
					i += 1;
				}

				unmap(self.data as *mut u8, 1);
			}
		}
	}
}

impl SlabAllocator {
	pub fn new(
		slab_size: usize,
		_max_free_slabs: usize,
		_max_total_slabs: usize,
		bitmap_pages: usize,
	) -> Result<Self, Error> {
		let bitmap = match BitMap::new(bitmap_pages) {
			Ok(bitmap) => bitmap,
			Err(e) => return Err(e),
		};

		let ret = Self {
			data: null_mut(),
			_head: null_mut(),
			_tail: null_mut(),
			bitmap,
			slab_size,
			_max_free_slabs,
			_max_total_slabs,
			lock: Lock::new(),
			_free_slabs: 0,
			_total_slabs: 0,
		};
		Ok(ret)
	}

	pub fn free(&mut self, slab: &Slab) {
		self.bitmap.free(slab.id);
	}

	pub fn alloc(&mut self) -> Result<Slab, Error> {
		let id = match self.bitmap.allocate() {
			Ok(id) => id,
			Err(e) => match e.kind {
				CapacityExceeded => match self.bitmap.extend() {
					Ok(_) => match self.bitmap.allocate() {
						Ok(id) => id,
						Err(e) => {
							return Err(e);
						}
					},
					Err(e) => {
						return Err(e);
					}
				},
				_ => {
					return Err(e);
				}
			},
		};

		let page_size = page_size!();
		let next = id >> unsafe { ctz(divide_usize(page_size, self.slab_size) as u32) };
		let k = next & mask!();
		let j = (next >> shift_per_level!()) & mask!();
		let i = (next >> 2 * shift_per_level!()) & mask!();

		let offset = offset!(id, self.slab_size);

		/*
		print!("i=");
		print_num!(i);
		print!(",j=");
		print_num!(j);
		print!(",k=");
		print_num!(k);
		print!(",off=");
		print_num!(offset);
		print!(",next=");
		print_num!(next);
		print!(",mask=");
		print_num!(mask!());
		println!("");
				*/

		unsafe {
			let mut lock = self.lock.read();

			if self.data.is_null() {
				lock.unlock();
				{
					let _lock = self.lock.write();
					if self.data.is_null() {
						self.data = map(1) as *mut *mut *mut *mut u8;
						if self.data.is_null() {
							return Err(err!(Alloc));
						}
					}
				}
				lock = self.lock.read();
			}

			if (*self.data.add(i)).is_null() {
				lock.unlock();
				{
					let _lock = self.lock.write();
					if (*self.data.add(i)).is_null() {
						*self.data.add(i) = map(1) as *mut *mut *mut u8;
						if (*self.data.add(i)).is_null() {
							return Err(err!(Alloc));
						}
					}
				}
				lock = self.lock.read();
			}

			if (*(*self.data.add(i)).add(j)).is_null() {
				lock.unlock();
				{
					let _lock = self.lock.write();
					if (*(*self.data.add(i)).add(j)).is_null() {
						*(*self.data.add(i)).add(j) = map(1) as *mut *mut u8;
						if (*(*self.data.add(i)).add(j)).is_null() {
							return Err(err!(Alloc));
						}
					}
				}
				lock = self.lock.read();
			}

			if (*(*(*self.data.add(i)).add(j)).add(k)).is_null() {
				lock.unlock();
				let _lock = self.lock.write();
				if (*(*(*self.data.add(i)).add(j)).add(k)).is_null() {
					*(*(*self.data.add(i)).add(j)).add(k) = map(1);
					if (*(*(*self.data.add(i)).add(j)).add(k)).is_null() {
						return Err(err!(Alloc));
					}
				}
			}
		}

		let ret = Slab {
			_next: null_mut(),
			id,
			data: unsafe { (*(*(*self.data.add(i)).add(j)).add(k)).add(offset) },
			len: self.slab_size,
		};
		Ok(ret)
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use core::mem::size_of;
	use core::slice::from_raw_parts_mut;

	#[test]
	fn test_slab() {
		let _start = getmicros!();
		let mut sa1 = SlabAllocator::new(128, 128, 256, 1).unwrap();
		let mut slab1 = sa1.alloc().unwrap();
		assert_eq!(slab1.id, 0);

		for i in 0..128 {
			slab1.get_mut()[i] = i as u8;
		}

		let mut slab2 = sa1.alloc().unwrap();
		assert_eq!(slab2.id, 1);
		for i in 0..128 {
			slab2.get_mut()[i] = (i + 1) as u8;
		}

		for i in 0..128 {
			assert_eq!(slab1.get()[i], i as u8);
		}
		for i in 0..128 {
			assert_eq!(slab2.get()[i], (i + 1) as u8);
		}
		/*
		print!("Micros: ");
		print_num!(getmicros!() - _start);
		println!("");
				*/
	}

	const SIZE: usize = 8;
	const COUNT: usize = 1024 * 1024;

	#[test]
	fn test_slab2() {
		let mut sa1 = SlabAllocator::new(SIZE, 128, 256, 20).unwrap();

		let pages_needed = 1 + divide_usize(COUNT * size_of::<Slab>(), page_size!());
		let slabs_ptr = unsafe { map(pages_needed) };
		let slabs = unsafe { from_raw_parts_mut(slabs_ptr as *mut Slab, COUNT) };

		let _start = getmicros!();
		for i in 0..COUNT {
			if i % (1024 * 1024) == 0 {
				if i != 0 {
					print!("loop: ");
					print_num!(i);
					println!("");
				}
			}
			slabs[i] = sa1.alloc().unwrap();
			assert_eq!(slabs[i].id, i);
			for j in 0..SIZE {
				slabs[i].get_mut()[j] = b'a' + ((i + j) % 26) as u8;
			}
		}

		for i in 0..COUNT {
			if i % (1024 * 1024) == 0 {
				if i != 0 {
					print!("free loop: ");
					print_num!(i);
					println!("");
				}
			}
			for j in 0..SIZE {
				assert_eq!(slabs[i].get()[j], b'a' + ((i + j) % 26) as u8);
			}
			sa1.free(&slabs[i]);
		}

		/*
		print!("micros=");
		print_num!(getmicros!() - _start);
		println!("");
				*/

		unsafe {
			unmap(slabs_ptr, pages_needed);
		}
	}

	// test malloc/free for comparison
	extern "C" {
		pub fn malloc(size: usize) -> *mut u8;
		pub fn free(ptr: *mut u8);
	}

	#[test]
	fn test_malloc() {
		let pages_needed = divide_usize(COUNT * size_of::<Slab>(), page_size!());
		let slabs_ptr = unsafe { map(pages_needed) };
		let slabs = unsafe { from_raw_parts_mut(slabs_ptr as *mut Slab, COUNT) };

		let _start = getmicros!();
		for i in 0..COUNT {
			if i % (1024 * 1024) == 0 {
				if i != 0 {
					print!("loop: ");
					print_num!(i);
					println!("");
				}
			}
			unsafe {
				slabs[i].data = malloc(SIZE);
			}
			slabs[i].len = SIZE;
			for j in 0..SIZE {
				slabs[i].get_mut()[j] = b'a' + ((i + j) % 26) as u8;
			}
		}

		for i in 0..COUNT {
			if i % (1024 * 1024) == 0 {
				if i != 0 {
					print!("free loop: ");
					print_num!(i);
					println!("");
				}
			}
			for j in 0..SIZE {
				assert_eq!(slabs[i].get()[j], b'a' + ((i + j) % 26) as u8);
			}
			unsafe {
				free(slabs[i].data);
			}
		}

		/*
		print!("micros=");
		print_num!(getmicros!() - _start);
		println!("");
				*/

		unsafe {
			unmap(slabs_ptr, pages_needed);
		}
	}
}
