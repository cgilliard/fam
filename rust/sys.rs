// system
extern "C" {
	pub fn read(fd: i32, buf: *mut u8, len: usize) -> i64;
	pub fn write(fd: i32, buf: *const u8, len: usize) -> i64;
	pub fn _exit(code: i32);
	pub fn map(pages: usize) -> *mut u8;
	pub fn unmap(ptr: *mut u8, pages: usize);
	pub fn getpagesize() -> i32;
}

// util
extern "C" {
	pub fn cstring_len(s: *const u8) -> usize;
	pub fn atomic_store_i64(ptr: *mut i64, value: i64);
	pub fn atomic_load_i64(ptr: *mut i64) -> i64;
	pub fn atomic_fetch_add_i64(ptr: *mut i64, value: i64) -> i64;
	pub fn atomic_fetch_sub_i64(ptr: *mut i64, value: i64) -> i64;
}

#[cfg(test)]
mod test {
	use aadd;
	use aload;
	use astore;
	use asub;

	#[test]
	fn test_sys() {
		let mut x: i64 = 1;
		aadd!(&mut x, 1);
		assert_eq!(aload!(&mut x), 2);
		asub!(&mut x, 1);
		assert_eq!(aload!(&mut x), 1);
		astore!(&mut x, 100);
		assert_eq!(aload!(&mut x), 100);
	}
}
