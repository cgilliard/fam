#![allow(dead_code)]

#[repr(C)]
pub struct MessageHeader {
	pub(crate) _next: *mut MessageHeader,
}

extern "C" {
	pub fn _exit(code: i32);
	pub fn write(fd: i32, buf: *const u8, len: usize) -> i64;
	pub fn alloc(len: usize) -> *mut u8;
	pub fn resize(ptr: *mut u8, len: usize) -> *mut u8;
	pub fn release(ptr: *mut u8);
	pub fn sleep_millis(millis: u64) -> i32;
	pub fn ptr_add(p: *mut u8, v: i64);
	pub fn getalloccount() -> i64;
	pub fn atomic_store_u64(ptr: *mut u64, value: u64);
	pub fn atomic_load_u64(ptr: *const u64) -> u64;
	pub fn atomic_fetch_add_u64(ptr: *mut u64, value: u64) -> u64;
	pub fn atomic_fetch_sub_u64(ptr: *mut u64, value: u64) -> u64;
	pub fn cas_release(ptr: *mut u64, expect: *const u64, desired: u64) -> bool;
	pub fn f64_to_str(d: f64, buf: *mut u8, capacity: u64) -> i32;
	pub fn sched_yield() -> i32;
	pub fn getmicros() -> i64;
	pub fn thread_create(start_routine: extern "C" fn(*mut u8), arg: *mut u8) -> i32;
	pub fn thread_create_joinable(
		handle: *const u8,
		start_routine: extern "C" fn(*mut u8),
		arg: *mut u8,
	) -> i32;
	pub fn thread_join(handle: *const u8) -> i32;
	pub fn thread_detach(handle: *const u8) -> i32;
	pub fn thread_handle_size() -> usize;

	pub fn channel_init(channel: *const u8) -> i32;
	pub fn channel_send(channel: *const u8, ptr: *const u8) -> i32;
	pub fn channel_recv(channel: *const u8, msg: *mut u8) -> *mut u8;
	pub fn channel_handle_size() -> usize;
	pub fn channel_destroy(channel: *const u8) -> i32;
	pub fn channel_pending(channel: *const u8) -> bool;
}

pub fn safe_channel_init(channel: *const u8) -> i32 {
	unsafe { channel_init(channel) }
}

pub fn safe_channel_send(channel: *const u8, ptr: *const u8) -> i32 {
	unsafe { channel_send(channel, ptr) }
}
pub fn safe_channel_recv(channel: *const u8, msg: *mut u8) -> *mut u8 {
	unsafe { channel_recv(channel, msg) }
}
pub fn safe_channel_handle_size() -> usize {
	unsafe { channel_handle_size() }
}
pub fn safe_channel_destroy(channel: *const u8) -> i32 {
	unsafe { channel_destroy(channel) }
}
pub fn safe_channel_pending(channel: *const u8) -> bool {
	unsafe { channel_pending(channel) }
}

pub fn safe_thread_create(start_routine: extern "C" fn(*mut u8), arg: *mut u8) -> i32 {
	unsafe { thread_create(start_routine, arg) }
}
pub fn safe_thread_create_joinable(
	handle: *const u8,
	start_routine: extern "C" fn(*mut u8),
	arg: *mut u8,
) -> i32 {
	unsafe { thread_create_joinable(handle, start_routine, arg) }
}
pub fn safe_thread_join(handle: *const u8) -> i32 {
	unsafe { thread_join(handle) }
}
pub fn safe_thread_detach(handle: *const u8) -> i32 {
	unsafe { thread_detach(handle) }
}
pub fn safe_thread_handle_size() -> usize {
	unsafe { thread_handle_size() }
}

pub fn safe_getmicros() -> i64 {
	unsafe { getmicros() }
}

pub fn safe_sched_yield() -> i32 {
	unsafe { sched_yield() }
}

pub fn safe_atomic_store_u64(ptr: *mut u64, value: u64) {
	unsafe { atomic_store_u64(ptr, value) }
}
pub fn safe_atomic_load_u64(ptr: *const u64) -> u64 {
	unsafe { atomic_load_u64(ptr) }
}
pub fn safe_atomic_fetch_add_u64(ptr: *mut u64, value: u64) -> u64 {
	unsafe { atomic_fetch_add_u64(ptr, value) }
}
pub fn safe_atomic_fetch_sub_u64(ptr: *mut u64, value: u64) -> u64 {
	unsafe { atomic_fetch_sub_u64(ptr, value) }
}

pub fn safe_cas_release(ptr: *mut u64, expect: *const u64, desired: u64) -> bool {
	unsafe { cas_release(ptr, expect, desired) }
}

pub fn safe_alloc(len: usize) -> *mut u8 {
	unsafe { alloc(len) }
}

pub fn safe_release(ptr: *mut u8) {
	unsafe { release(ptr) }
}

pub fn safe_resize(ptr: *mut u8, len: usize) -> *mut u8 {
	unsafe { resize(ptr, len) }
}

pub fn safe_write(fd: i32, buf: *const u8, len: usize) -> i64 {
	unsafe { write(fd, buf, len) }
}

pub fn safe_exit(code: i32) {
	unsafe {
		_exit(code);
	}
}
pub fn safe_f64_to_str(d: f64, buf: *mut u8, capacity: u64) -> i32 {
	unsafe { f64_to_str(d, buf, capacity) }
}

pub fn safe_sleep_millis(millis: u64) -> i32 {
	unsafe { sleep_millis(millis) }
}

pub fn safe_ptr_add(p: *mut u8, v: i64) {
	unsafe { ptr_add(p, v) }
}

#[allow(dead_code)]
pub fn safe_getalloccount() -> i64 {
	unsafe { getalloccount() }
}
