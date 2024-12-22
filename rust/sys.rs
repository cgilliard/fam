#[repr(C)]
pub struct Message {
	pub(crate) _next: *mut Message,
	pub payload: *mut u8,
	pub spo: [u8; 48],
}

extern "C" {
	//pub fn read(fd: i32, buf: *mut u8, len: usize) -> i64;
	//pub fn sleep(duration: u64) -> i32;
	pub fn write(fd: i32, buf: *const u8, len: usize) -> i64;
	pub fn _exit(code: i32);
	pub fn getpagesize() -> i32;
	pub fn sched_yield() -> i32;
	//pub fn getmicros() -> u64;
	pub fn thread_create(start_routine: extern "C" fn(*mut u8), arg: *mut u8) -> i32;
	pub fn channel_init(channel: *const u8) -> i32;
	pub fn channel_send(channel: *const u8, ptr: *const u8) -> i32;
	pub fn channel_recv(channel: *const u8) -> *mut u8;
	pub fn channel_handle_size() -> usize;
	pub fn channel_destroy(channel: *const u8) -> i32;
	pub fn atomic_store_u64(ptr: *mut u64, value: u64);
	pub fn atomic_load_u64(ptr: *const u64) -> u64;
	pub fn atomic_fetch_add_u64(ptr: *mut u64, value: u64) -> u64;
	pub fn atomic_fetch_sub_u64(ptr: *mut u64, value: u64) -> u64;
	pub fn cas_release(ptr: *mut u64, expect: *const u64, desired: u64) -> bool;
	//pub fn ctzl(v: u64) -> i32;
	//pub fn ctz(v: u32) -> i32;
	pub fn getalloccount() -> i64;
	pub fn alloc(len: usize) -> *mut u8;
	pub fn release(ptr: *mut u8);
}

#[cfg(test)]
mod test {
	use super::*;
	use core::mem::size_of;

	extern "C" fn test_thread(channel: *mut u8) {
		unsafe {
			let msg = alloc(size_of::<Message>()) as *mut Message;
			let payload = alloc(8);
			*(payload.add(0)) = b'a';
			*(payload.add(1)) = b'b';
			*(payload.add(2)) = b'c';
			(*msg).payload = payload;
			(*msg).spo[0] = b'd';
			(*msg).spo[1] = b'e';
			(*msg).spo[2] = b'f';
			channel_send(channel, msg as *mut u8);
		}
	}

	#[test]
	fn test_channel_sys() {
		unsafe {
			assert!(channel_handle_size() < getpagesize() as usize);
			let channel = alloc(channel_handle_size());
			channel_init(channel);
			thread_create(test_thread, channel);
			let recv = channel_recv(channel) as *mut Message;
			assert_eq!(*(*recv).payload.add(0), b'a');
			assert_eq!(*(*recv).payload.add(1), b'b');
			assert_eq!(*(*recv).payload.add(2), b'c');
			assert_eq!((*recv).spo[0], b'd');
			assert_eq!((*recv).spo[1], b'e');
			assert_eq!((*recv).spo[2], b'f');
			channel_destroy(channel);
			release(recv as *mut u8);
			release(channel);

			let channel = alloc(channel_handle_size());
			channel_init(channel);
			thread_create(test_thread, channel);
			let recv = channel_recv(channel) as *mut Message;
			assert_eq!(*(*recv).payload.add(0), b'a');
			assert_eq!(*(*recv).payload.add(1), b'b');
			assert_eq!(*(*recv).payload.add(2), b'c');
			assert_eq!((*recv).spo[0], b'd');
			assert_eq!((*recv).spo[1], b'e');
			assert_eq!((*recv).spo[2], b'f');
			channel_destroy(channel);
			release((*recv).payload);
			release(recv as *mut u8);
			release(channel);
		}
	}
}
