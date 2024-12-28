use core::mem::size_of;
use core::slice::from_raw_parts;
use util::murmur::{murmur3_32_of_slice, MURMUR_SEED};

pub use core::cmp::PartialEq;

pub trait Ord {
	fn compare(&self, other: &Self) -> i8;
}

pub trait Hash {
	fn hash(&self) -> usize;
}

impl Ord for i32 {
	fn compare(&self, other: &Self) -> i8 {
		if *self < *other {
			-1
		} else if *self > *other {
			1
		} else {
			0
		}
	}
}

impl Ord for u64 {
	fn compare(&self, other: &Self) -> i8 {
		if *self < *other {
			-1
		} else if *self > *other {
			1
		} else {
			0
		}
	}
}

impl Hash for i32 {
	fn hash(&self) -> usize {
		let slice = unsafe { from_raw_parts(self as *const i32 as *const u8, size_of::<i32>()) };
		murmur3_32_of_slice(slice, MURMUR_SEED) as usize
	}
}
