use core::mem::size_of;
use core::slice::from_raw_parts;
use prelude::*;
use sys::safe_rand_bytes;

pub trait Display {
	fn format(&self, f: &mut Formatter) -> Result<(), Error>;
}

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

static mut STATIC_MURMUR_SEED: u64 = 0u64;

#[allow(static_mut_refs)]
pub fn get_murmur_seed() -> u32 {
	unsafe {
		loop {
			let cur = aload!(&STATIC_MURMUR_SEED);
			if cur != 0 {
				return cur as u32;
			}
			let mut nval = 0u64;
			safe_rand_bytes(&mut nval as *mut u64 as *mut u8, size_of::<u64>());
			if nval == 0 {
				continue;
			}
			if cas!(&mut STATIC_MURMUR_SEED, &cur, nval) {
				return nval as u32;
			}
		}
	}
}

impl Hash for i32 {
	fn hash(&self) -> usize {
		let slice = unsafe { from_raw_parts(self as *const i32 as *const u8, size_of::<i32>()) };
		murmur3_32_of_slice(slice, get_murmur_seed()) as usize
	}
}
