#[repr(C)]
pub struct Nano {
	pub high: u64,
	pub low: u64,
}

impl Nano {
	pub fn to_u128(&self) -> u128 {
		(self.high as u128) << 64 | self.low as u128
	}
}

// system
extern "C" {
	pub fn map(pages: u64) -> *mut u8;
	pub fn unmap(ptr: *mut u8, pages: u64);
	pub fn getpagesize() -> i32;
	pub fn fmap(offset: u64) -> *mut u8;
	pub fn read(fd: i32, buf: *mut u8, len: usize) -> i64;
	pub fn write(fd: i32, buf: *const u8, len: usize) -> i64;
	pub fn _exit(code: i32);
	pub fn os_sleep(millis: u64) -> i32;
	pub fn getnanos() -> Nano;
}

// utils
extern "C" {
	pub fn cstring_len(buf: *const u8) -> u64;
	pub fn cstring_compare(s1: *const u8, s2: *const u8) -> i32;
	pub fn cstring_compare_n(s1: *const u8, s2: *const u8, n: u64) -> i32;
}

macro_rules! pages {
	($v:expr) => {{
		use sys::getpagesize;
		let size = unsafe { getpagesize() };
		if size > 0 {
			1 + ($v as u64 - 1 as u64) / size as u64
		} else {
			0
		}
	}};
}

#[macro_export]
macro_rules! page_size {
	() => {{
		use sys::getpagesize;
		let v = unsafe { getpagesize() } as u64;
		v
	}};
}

#[macro_export]
macro_rules! panic {
	($s:expr) => {{
		use sys::{_exit, cstring_len, write};
		unsafe {
			let sptr = $s.as_ptr();
			write(2, sptr, cstring_len(sptr) as usize);
			write(2, "\n\0".as_ptr(), 2);
			_exit(-1);
		}
	}};
}

#[cfg(test)]
mod test {
	use super::*;
	//use crate::println;

	#[test]
	fn test_map() {
		unsafe {
			let x = map(1);
			unmap(x, 1);
			//	println!("abc {}, {} {}", 1, "something", 1.5);
		}
	}
}
