use core::ops::Drop;
use core::result::{Result, Result::Err, Result::Ok};
use core::slice::from_raw_parts_mut;
use err;
use std::bitmap::BitMap;
use std::error::{Error, ErrorKind::Alloc};
use sys::ctz;

macro_rules! shift_per_level {
	($z:expr) => {{
		unsafe { ctz($z) }
	}};
}

macro_rules! mask {
	($z:expr) => {{
		((0x1 << (shift_per_level!($z))) - 1)
	}};
}

macro_rules! koff {
	($z:expr) => {{
		shift_per_level!($z)
	}};
}

macro_rules! joff {
	($z:expr) => {{
		2 * shift_per_level!($z)
	}};
}

macro_rules! ioff {
	($z:expr) => {{
		3 * shift_per_level!($z)
	}};
}

pub struct SlabAllocator {
	data: *mut *mut *mut *mut u8,
	_bitmap: BitMap,
}

pub struct Slab<'a> {
	data: &'a mut [u8],
	id: u64,
}

impl Slab<'_> {
	pub fn get(&self) -> &[u8] {
		&self.data
	}

	pub fn get_mut(&mut self) -> &mut [u8] {
		&mut self.data
	}
}

impl Drop for SlabAllocator {
	fn drop(&mut self) {}
}

impl SlabAllocator {
	pub fn new(_slab_size: usize, _max_free: usize, _max_total: usize) -> Result<Self, Error> {
		let _ = koff!(0);
		let _ = joff!(0);
		let _ = ioff!(0);
		let _ = mask!(0);
		Err(err!(Alloc))
	}

	pub fn alloc(&mut self) -> Result<Slab, Error> {
		let id = 0;
		let len = 12;
		Ok(Slab {
			id,
			data: unsafe { from_raw_parts_mut(***self.data, len) },
		})
	}

	pub fn free(&mut self, slab: &Slab) {
		let _ = slab.id;
	}
}
