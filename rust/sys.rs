// system
extern "C" {
	//pub fn read(fd: i32, buf: *mut u8, len: usize) -> i64;
	//pub fn sleep(duration: u64) -> i32;
	pub fn write(fd: i32, buf: *const u8, len: usize) -> i64;
	pub fn _exit(code: i32);
	pub fn map(pages: usize) -> *mut u8;
	pub fn unmap(ptr: *mut u8, pages: usize);
	pub fn getpagesize() -> i32;
	pub fn sched_yield() -> i32;
	pub fn getmicros() -> u64;
	pub fn thread_create(
		handle: *mut u8,
		start_routine: extern "C" fn(*mut u64) -> *mut u64,
		arg: *mut u64,
	) -> i32;
	pub fn thread_join(handle: *mut u8) -> i32;
	pub fn thread_detach(handle: *mut u8) -> i32;
	pub fn thread_handle_size() -> usize;
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
	#![allow(static_mut_refs)]
	use super::*;
	use aadd;
	use aload;
	use astore;
	use asub;
	use cas;
	use core::ptr::null_mut;
	use page_size;

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

	static mut GLOBAL_COUNT: u64 = 0;

	extern "C" fn start_thread(arg: *mut u64) -> *mut u64 {
		unsafe {
			assert_eq!(*arg, 71);
		}
		aadd!(&mut GLOBAL_COUNT, 1);
		null_mut()
	}

	#[test]
	fn test_thread() {
		assert!(unsafe { thread_handle_size() } <= page_size!());
		let handle = unsafe { map(1) };
		let ptr = unsafe { map(1) } as *mut u64;
		unsafe {
			*ptr = 71u64;
		}
		assert_eq!(aload!(&GLOBAL_COUNT), 0);
		let res = unsafe { thread_create(handle, start_thread, ptr) };
		assert_eq!(res, 0);
		unsafe {
			thread_join(handle);
			unmap(handle, 1);
			unmap(ptr as *mut u8, 1);
		}
		assert_eq!(aload!(&GLOBAL_COUNT), 1);
	}
}
