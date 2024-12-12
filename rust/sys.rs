// system
extern "C" {
	pub fn read(fd: i32, buf: *mut u8, len: usize) -> i64;
	pub fn write(fd: i32, buf: *const u8, len: usize) -> i64;
	pub fn _exit(code: i32);
}

fn test1(x: bool) -> i32 {
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
	assert_eq!(test1(false), 1);
	assert_eq!(test1(true), 0); // Test both branches
}