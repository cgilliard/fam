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

extern "C" {
	pub fn map(pages: u64) -> *mut u8;
	pub fn unmap(ptr: *mut u8, pages: u64);
	pub fn getpagesize() -> i32;
	pub fn fmap(offset: u64) -> *mut u8;
	pub fn read(fd: i32, buf: *mut u8, len: u64) -> i64;
	pub fn write(fd: i32, buf: *const u8, len: u64) -> i64;
	pub fn _exit(code: i32);
	pub fn os_sleep(millis: u64) -> i32;
	pub fn getnanos() -> Nano;
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::println;
	#[test]
	fn test_map() {
		unsafe {
			let x = map(1);
			unmap(x, 1);
			println!(x);
		}
	}
}
