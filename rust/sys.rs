// system
extern "C" {
        //pub fn read(fd: i32, buf: *mut u8, len: usize) -> i64;
        //pub fn sleep(duration: u64) -> i32;
        pub fn write(fd: i32, buf: *const u8, len: usize) -> i64;
        pub fn _exit(code: i32);
}
