// system
extern "C" {
	pub fn read(fd: i32, buf: *mut u8, len: usize) -> i64;
	pub fn write(fd: i32, buf: *const u8, len: usize) -> i64;
	pub fn _exit(code: i32);
	pub fn map(pages: usize) -> *mut u8;
	pub fn unmap(ptr: *mut u8, pages: usize);
	pub fn getpagesize() -> i32;
	pub fn sched_yield() -> i32;
}

// util
extern "C" {
	pub fn cstring_len(s: *const u8) -> usize;
	pub fn atomic_store_u64(ptr: *mut u64, value: u64);
	pub fn atomic_load_u64(ptr: *const u64) -> u64;
	pub fn atomic_fetch_add_u64(ptr: *mut u64, value: u64) -> u64;
	pub fn atomic_fetch_sub_u64(ptr: *mut u64, value: u64) -> u64;
	pub fn cas_release(ptr: *mut u64, expect: *const u64, desired: u64) -> bool;
	pub fn ctzl(v: u64) -> i32;
	pub fn ctz(v: u32) -> i32;
}

#[cfg(test)]
mod test {
	use aadd;
	use aload;
	use astore;
	use asub;
	use cas;

	#[test]
	fn test_sys() {
		let mut x: u64 = 1;
		aadd!(&mut x, 1);
		assert_eq!(aload!(&mut x), 2);
		asub!(&mut x, 1);
		assert_eq!(aload!(&mut x), 1);
		astore!(&mut x, 100);
		assert_eq!(aload!(&mut x), 100);
		let mut x = 0u64;
		let mut y = 0u64;

		assert!(cas!(&mut x, &mut y, 10));
		assert_eq!(x, 10);

		x = 0u64;
		y = 1u64;
		assert!(!cas!(&mut x, &mut y, 10));
		assert_eq!(x, 0);
	}
}
