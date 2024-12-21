#[repr(C)]
pub struct Message {
	_next: *mut Message,
	pub payload: *mut u8,
}

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
		start_routine: extern "C" fn(*mut u8) -> *mut u8,
		arg: *mut u8,
		detached: bool,
	) -> i32;
	pub fn thread_join(handle: *mut u8) -> i32;
	pub fn thread_detach(handle: *mut u8) -> i32;
	pub fn thread_handle_size() -> usize;
	pub fn channel_init(channel: *mut u8) -> i32;
	pub fn channel_send(channel: *mut u8, ptr: *mut u8) -> i32;
	pub fn channel_recv(channel: *mut u8) -> *mut u8;
	pub fn channel_handle_size() -> usize;
	pub fn channel_destroy(channel: *mut u8) -> i32;
}

// util
extern "C" {
	pub fn atomic_store_u64(ptr: *mut u64, value: u64);
	pub fn atomic_load_u64(ptr: *const u64) -> u64;
	pub fn atomic_fetch_add_u64(ptr: *mut u64, value: u64) -> u64;
	pub fn atomic_fetch_sub_u64(ptr: *mut u64, value: u64) -> u64;
	pub fn cas_release(ptr: *mut u64, expect: *const u64, desired: u64) -> bool;
	pub fn ctzl(v: u64) -> i32;
	pub fn ctz(v: u32) -> i32;
	pub fn getalloccount() -> i64;
}

#[cfg(test)]
mod test {
	use super::*;
	use core::ptr::null_mut;

	extern "C" fn test_thread(channel: *mut u8) -> *mut u8 {
		unsafe {
			let msg = map(1) as *mut Message;
			let payload = map(1);
			*(payload.add(0)) = b'a';
			*(payload.add(1)) = b'b';
			*(payload.add(2)) = b'c';
			(*msg).payload = payload;
			channel_send(channel, msg as *mut u8);
		}
		null_mut()
	}

	#[test]
	fn test_channel_sys() {
		unsafe {
			assert!(channel_handle_size() < getpagesize() as usize);
			assert!(thread_handle_size() < getpagesize() as usize);
			let channel = map(1);
			let handle = map(1);
			channel_init(channel);
			thread_create(handle, test_thread, channel, false);
			let recv = channel_recv(channel) as *mut Message;
			assert_eq!(*(*recv).payload.add(0), b'a');
			assert_eq!(*(*recv).payload.add(1), b'b');
			assert_eq!(*(*recv).payload.add(2), b'c');
			thread_join(handle);
			channel_destroy(channel);
			unmap(recv as *mut u8, 1);
			unmap(channel, 1);
			unmap(handle, 1);

			let channel = map(1);
			let handle = map(1);
			channel_init(channel);
			thread_create(handle, test_thread, channel, false);
			let recv = channel_recv(channel) as *mut Message;
			assert_eq!(*(*recv).payload.add(0), b'a');
			assert_eq!(*(*recv).payload.add(1), b'b');
			assert_eq!(*(*recv).payload.add(2), b'c');
			thread_detach(handle);
			channel_destroy(channel);
			unmap((*recv).payload, 1);
			unmap(recv as *mut u8, 1);
			unmap(channel, 1);
			unmap(handle, 1);
		}
	}
}
