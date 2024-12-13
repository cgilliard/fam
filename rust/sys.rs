// system
extern "C" {
	pub fn read(fd: i32, buf: *mut u8, len: usize) -> i64;
	pub fn write(fd: i32, buf: *const u8, len: usize) -> i64;
	pub fn _exit(code: i32);
	pub fn map(pages: u64) -> *mut u8;
	pub fn unmap(ptr: *mut u8, pages: u64);
	pub fn getpagesize() -> i32;
}

// util
extern "C" {
	pub fn cstring_len(s: *const u8) -> usize;
}

fn _test1(x: bool) -> i32 {
	// Take x as an argument
	let y;
	if x {
		y = 0;
	} else {
		y = 1;
	}
	y
}

#[test]
fn test_sys() {
	assert_eq!(_test1(false), 1);
	assert_eq!(_test1(true), 0); // Test both branches
}
