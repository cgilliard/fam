#![allow(dead_code)]

#[repr(C)]
pub struct MessageHeader {
	pub(crate) _reserved: *mut u8,
}

extern "C" {
	pub fn _exit(code: i32);
	pub fn write(fd: i32, buf: *const u8, len: usize) -> i64;
	pub fn alloc(len: usize) -> *const u8;
	pub fn resize(ptr: *const u8, len: usize) -> *const u8;
	pub fn release(ptr: *const u8);
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

	pub fn channel_init(channel: *const u8, msg_size: u64, capacity: u64) -> i32;
	pub fn channel_send(channel: *const u8, ptr: *const u8) -> i32;
	pub fn channel_recv(channel: *const u8, msg: *mut u8) -> *mut u8;
	pub fn channel_handle_size() -> usize;
	pub fn channel_destroy(channel: *const u8);

	pub fn socket_handle_size() -> usize;
	pub fn socket_event_size() -> usize;
	pub fn socket_multiplex_handle_size() -> usize;
	pub fn socket_connect(handle: *mut u8, addr: *const u8, port: i32) -> i32;
	pub fn socket_shutdown(handle: *const u8) -> i32;
	pub fn socket_close(handle: *const u8) -> i32;
	pub fn socket_listen(handle: *mut u8, addr: *const u8, port: u16, backlog: i32) -> i32;
	pub fn socket_accept(handle: *const u8, nhandle: *mut u8) -> i32;
	pub fn socket_send(handle: *const u8, buf: *const u8, len: usize) -> i64;
	pub fn socket_recv(handle: *const u8, buf: *mut u8, capacity: usize) -> i64;

	pub fn socket_multiplex_init(handle: *mut u8) -> i32;
	pub fn socket_multiplex_register(handle: *const u8, socket: *const u8, flags: i32) -> i32;
	pub fn socket_multiplex_wait(handle: *const u8, events: *mut u8, max_events: i32) -> i32;
	pub fn socket_event_handle(handle: *mut u8, event: *const u8);
	pub fn socket_event_is_read(event: *const u8) -> bool;
	pub fn socket_event_is_write(event: *const u8) -> bool;
	pub fn socket_handle_eq(handle1: *const u8, handle2: *const u8) -> bool;

	pub fn pipe(pair: *mut u8) -> i32;
	pub fn Base64decode(output: *mut u8, input: *mut u8);
	pub fn Base64encode(input: *const u8, output: *mut u8, len: usize);
	pub fn SHA1(data: *const u8, size: usize, hash: *mut u8);

	pub fn rand_bytes(data: *mut u8, len: usize);

}

pub fn safe_rand_bytes(data: *mut u8, len: usize) {
	unsafe { rand_bytes(data, len) }
}

pub fn safe_pipe(pair: *mut u8) -> i32 {
	unsafe { pipe(pair) }
}

pub fn safe_socket_handle_size() -> usize {
	unsafe { socket_handle_size() }
}
pub fn safe_socket_event_size() -> usize {
	unsafe { socket_event_size() }
}
pub fn safe_socket_multiplex_handle_size() -> usize {
	unsafe { socket_multiplex_handle_size() }
}
pub fn safe_socket_connect(handle: *mut u8, addr: *const u8, port: i32) -> i32 {
	unsafe { socket_connect(handle, addr, port) }
}
pub fn safe_socket_shutdown(handle: *const u8) -> i32 {
	unsafe { socket_shutdown(handle) }
}
pub fn safe_socket_close(handle: *const u8) -> i32 {
	unsafe { socket_close(handle) }
}
pub fn safe_socket_listen(handle: *mut u8, addr: *const u8, port: u16, backlog: i32) -> i32 {
	unsafe { socket_listen(handle, addr, port, backlog) }
}
pub fn safe_socket_accept(handle: *const u8, nhandle: *mut u8) -> i32 {
	unsafe { socket_accept(handle, nhandle) }
}
pub fn safe_socket_send(handle: *const u8, buf: *const u8, len: usize) -> i64 {
	unsafe { socket_send(handle, buf, len) }
}
pub fn safe_socket_recv(handle: *const u8, buf: *mut u8, capacity: usize) -> i64 {
	unsafe { socket_recv(handle, buf, capacity) }
}

pub fn safe_socket_multiplex_init(handle: *mut u8) -> i32 {
	unsafe { socket_multiplex_init(handle) }
}
pub fn safe_socket_multiplex_register(handle: *const u8, socket: *const u8, flags: i32) -> i32 {
	unsafe { socket_multiplex_register(handle, socket, flags) }
}
pub fn safe_socket_multiplex_wait(handle: *const u8, events: *mut u8, max_events: i32) -> i32 {
	unsafe { socket_multiplex_wait(handle, events, max_events) }
}
pub fn safe_socket_event_handle(handle: *mut u8, event: *const u8) {
	unsafe { socket_event_handle(handle, event) }
}
pub fn safe_socket_event_is_read(event: *const u8) -> bool {
	unsafe { socket_event_is_read(event) }
}
pub fn safe_socket_event_is_write(event: *const u8) -> bool {
	unsafe { socket_event_is_write(event) }
}
pub fn safe_socket_handle_eq(handle1: *const u8, handle2: *const u8) -> bool {
	unsafe { socket_handle_eq(handle1, handle2) }
}

pub fn safe_channel_init(channel: *const u8, msg_size: u64, capacity: u64) -> i32 {
	unsafe { channel_init(channel, msg_size, capacity) }
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
pub fn safe_channel_destroy(channel: *const u8) {
	unsafe {
		channel_destroy(channel);
	}
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

pub fn safe_alloc(len: usize) -> *const u8 {
	unsafe { alloc(len) }
}

pub fn safe_release(ptr: *const u8) {
	unsafe { release(ptr) }
}

pub fn safe_resize(ptr: *const u8, len: usize) -> *const u8 {
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
