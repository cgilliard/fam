use core::ptr::null_mut;

#[repr(C)]
#[allow(dead_code)]
pub struct Message {
	pub(crate) _next: *mut Message,
	pub payload: *mut u8,
}

impl Message {
	#[allow(dead_code)]
	pub fn empty() -> Self {
		Self {
			_next: null_mut(),
			payload: null_mut(),
		}
	}
}

#[repr(C)]
pub struct MessageHeader {
	pub(crate) _next: *mut MessageHeader,
}

#[repr(C)]
#[allow(dead_code)]
pub struct BoundedMessage {
	pub(crate) len: u64,
}

#[allow(dead_code)]
extern "C" {
	pub fn read(fd: i32, buf: *mut u8, len: usize) -> i64;
	pub fn write(fd: i32, buf: *const u8, len: usize) -> i64;
	pub fn _exit(code: i32);
	pub fn getpagesize() -> i32;
	pub fn sched_yield() -> i32;
	pub fn getmicros() -> u64;
	pub fn backtrace_full() -> *const u8;
	pub fn thread_create(start_routine: extern "C" fn(*mut u8), arg: *mut u8) -> i32;
	pub fn thread_create_joinable(
		handle: *const u8,
		start_routine: extern "C" fn(*mut u8),
		arg: *mut u8,
	) -> i32;
	pub fn thread_join(handle: *const u8) -> i32;
	pub fn thread_detach(handle: *const u8) -> i32;
	pub fn thread_handle_size() -> usize;
	pub fn channel_unbounded_init(channel: *const u8) -> i32;
	pub fn channel_bounded_init(channel: *const u8, capacity: usize, msg_size: usize) -> i32;
	pub fn channel_send(channel: *const u8, ptr: *const u8) -> i32;
	pub fn channel_recv(channel: *const u8, msg: *mut u8) -> *mut u8;
	pub fn channel_handle_size() -> usize;
	pub fn channel_destroy(channel: *const u8) -> i32;
	pub fn channel_pending(channel: *const u8) -> bool;
	pub fn atomic_store_u64(ptr: *mut u64, value: u64);
	pub fn atomic_load_u64(ptr: *const u64) -> u64;
	pub fn atomic_fetch_add_u64(ptr: *mut u64, value: u64) -> u64;
	pub fn atomic_fetch_sub_u64(ptr: *mut u64, value: u64) -> u64;
	pub fn cas_release(ptr: *mut u64, expect: *const u64, desired: u64) -> bool;
	pub fn getalloccount() -> i64;
	pub fn alloc(len: usize) -> *mut u8;
	pub fn resize(ptr: *mut u8, len: usize) -> *mut u8;
	pub fn release(ptr: *mut u8);
	pub fn sleep_millis(millis: u64) -> i32;
	pub fn f64_to_str(d: f64, buf: *mut u8, capacity: u64) -> i32;
	pub fn ptr_add(p: *mut u8, v: i64);

	pub fn socket_handle_size() -> usize;
	pub fn socket_event_size() -> usize;
	pub fn socket_multiplex_handle_size() -> usize;
	pub fn socket_connect(handle: *mut u8, addr: *const u8, port: i32) -> i32;
	pub fn socket_shutdown(handle: *mut u8) -> i32;
	pub fn socket_close(handle: *mut u8) -> i32;
	pub fn socket_listen(handle: *mut u8, addr: *const u8, port: u16, backlog: i32) -> i32;
	pub fn socket_accept(handle: *mut u8, nhandle: *mut u8) -> i32;
	pub fn socket_send(handle: *mut u8, buf: *const u8, len: usize) -> i64;
	pub fn socket_recv(handle: *mut u8, buf: *const u8, capacity: usize) -> i64;

	pub fn socket_multiplex_init(handle: *mut u8) -> i32;
	pub fn socket_multiplex_register(handle: *mut u8, socket: *mut u8, flags: i32) -> i32;
	pub fn socket_multiplex_wait(handle: *mut u8, events: *mut u8, max_events: i32) -> i32;
	pub fn socket_event_handle(handle: *mut u8, event: *mut u8);
	pub fn socket_event_is_read(event: *mut u8) -> bool;
	pub fn socket_event_is_write(event: *mut u8) -> bool;
	pub fn socket_fd(handle: *mut u8) -> i32;
	pub fn rand_bytes(buf: *mut u8, len: u64) -> i32;
}

pub fn safe_write(fd: i32, buf: *const u8, len: usize) -> i64 {
	unsafe { write(fd, buf, len) }
}

pub fn safe_exit(code: i32) {
	unsafe {
		_exit(code);
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use core::mem::size_of;
	use core::slice::from_raw_parts;
	use core::str::from_utf8_unchecked;
	use prelude::*;

	extern "C" fn test_thread(channel: *mut u8) {
		unsafe {
			let msg = alloc(size_of::<MessageHeader>() + 8);
			*(msg.add(size_of::<MessageHeader>())) = b'a';
			*(msg.add(size_of::<MessageHeader>() + 1)) = b'b';
			*(msg.add(size_of::<MessageHeader>() + 2)) = b'c';
			channel_send(channel, msg as *mut u8);
		}
	}

	#[test]
	fn test_ptr_add() {
		unsafe {
			let mut v = alloc(128);
			*v = 50;
			assert_eq!(*v, 50);
			v = v.add(1);
			*v = 51;
			assert_eq!(*v, 51);
			v = v.sub(1);
			assert_eq!(*v, 50);
			ptr_add(v, 1);
			assert_eq!(*v, 51);
			ptr_add(v, -1);
			assert_eq!(*v, 50);
			release(v);
		}
	}

	#[test]
	fn test_channel_sys() {
		let initial = unsafe { getalloccount() };
		unsafe {
			let channel = alloc(channel_handle_size());
			channel_unbounded_init(channel);
			thread_create(test_thread, channel);
			let mut msg: Message = Message::empty();
			let recv = channel_recv(channel, &mut msg as *mut Message as *mut u8) as *mut u8;
			assert_eq!(*recv.add(size_of::<MessageHeader>()), b'a');
			assert_eq!(*recv.add(size_of::<MessageHeader>() + 1), b'b');
			assert_eq!(*recv.add(size_of::<MessageHeader>() + 2), b'c');
			channel_destroy(channel);
			release(recv as *mut u8);
			release(channel);
		}

		assert_eq!(initial, unsafe { getalloccount() });
	}

	#[repr(C)]
	struct BoundedMessageU64 {
		header: BoundedMessage,
		value: u64,
	}

	impl BoundedMessageU64 {
		fn empty() -> Self {
			Self {
				header: BoundedMessage {
					len: size_of::<u64>() as u64,
				},
				value: 0,
			}
		}
	}

	#[test]
	fn test_bounded_channels_sys_stack() {
		let initial = unsafe { getalloccount() };
		unsafe {
			let channel = alloc(channel_handle_size() + size_of::<u64>() * 100);
			assert!(!channel.is_null());
			assert!(channel_bounded_init(channel, 100, size_of::<u64>()) == 0);

			let msg = BoundedMessageU64 {
				header: BoundedMessage {
					len: size_of::<u64>() as u64,
				},
				value: 1234,
			};

			let msg_ptr = &msg as *const BoundedMessageU64 as *const u8;
			assert!(channel_send(channel, msg_ptr as *mut u8) == 0);

			let mut msg: BoundedMessageU64 = BoundedMessageU64::empty();
			let recv_msg = channel_recv(channel, &mut msg as *mut BoundedMessageU64 as *mut u8);
			assert!(!recv_msg.is_null());

			let recv_payload = recv_msg as *const BoundedMessageU64;
			assert_eq!((*recv_payload).value, 1234);

			for _i in 0..99 {
				let msg_ptr = &msg as *const BoundedMessageU64 as *const u8;
				assert!(channel_send(channel, msg_ptr as *mut u8) == 0);
			}
			let msg_ptr = &msg as *const BoundedMessageU64 as *const u8;
			assert!(channel_send(channel, msg_ptr as *mut u8) != 0);

			for _i in 0..99 {
				let recv_msg = channel_recv(channel, &mut msg as *mut BoundedMessageU64 as *mut u8);
				assert!(!recv_msg.is_null());
				let recv_payload = recv_msg as *const BoundedMessageU64;
				assert_eq!((*recv_payload).value, 1234);
			}

			release(channel); // Clean up the channel
		}
		unsafe {
			assert_eq!(initial, getalloccount());
		}
	}

	#[test]
	fn test_sock_sys() {
		unsafe {
			let addr: [u8; 4] = [127, 0, 0, 1];
			let server = alloc(socket_handle_size());
			let client = alloc(socket_handle_size());
			let accepted = alloc(socket_handle_size());
			let port = socket_listen(server, addr.as_ptr(), 0, 10);
			assert_eq!(socket_connect(client, addr.as_ptr(), port), 0);
			assert_eq!(socket_accept(server, accepted), 0);
			let buf: [u8; 1] = [b'h'];
			let mut recv_buf = [0u8; 1];
			/*
			assert_eq!(socket_send(client, buf.as_ptr(), 1), 1);
			assert_eq!(recv_buf, [0u8; 1]);
			assert_eq!(socket_recv(accepted, recv_buf.as_mut_ptr(), 1), 1);
			assert_eq!(recv_buf, buf);
						*/

			assert_eq!(socket_close(server), 0);
			assert_eq!(socket_close(client), 0);
			assert_eq!(socket_close(accepted), 0);

			release(server);
			release(client);
			release(accepted);
		}
	}

	#[test]
	fn test_sock_stack_sys() {
		unsafe {
			let addr: [u8; 4] = [127, 0, 0, 1];

			// Create raw pointers for the sockets
			let mut server_i32 = [0u8; 4];
			let mut client_i32 = [0u8; 4];
			let mut accepted_i32 = [0u8; 4];

			let server: *mut u8 = &mut server_i32 as *mut u8;
			let client: *mut u8 = &mut client_i32 as *mut u8;
			let accepted: *mut u8 = &mut accepted_i32 as *mut u8;

			// Initialize the server, client, and accepted socket handles
			let port = socket_listen(server, addr.as_ptr(), 0, 10);
			assert_eq!(socket_connect(client, addr.as_ptr(), port), 0);
			assert_eq!(socket_accept(server, accepted), 0);

			let buf: [u8; 1] = [b'h'];
			let mut recv_buf = [0u8; 1];

			// Send and receive data
			/*
			assert_eq!(socket_send(client, buf.as_ptr(), 1), 1);
			assert_eq!(recv_buf, [0u8; 1]);
			assert_eq!(socket_recv(accepted, recv_buf.as_mut_ptr(), 1), 1);
			assert_eq!(recv_buf, buf);
						*/

			// Close the sockets
			assert_eq!(socket_close(server), 0);
			assert_eq!(socket_close(client), 0);
			assert_eq!(socket_close(accepted), 0);
		}
	}

	#[test]
	fn test_multiplex() {
		unsafe {
			// init addr to localhost
			let addr: [u8; 4] = [127, 0, 0, 1];
			let events = alloc(socket_event_size() * 10);
			let server = alloc(socket_handle_size());
			let client = alloc(socket_handle_size());
			let accepted = alloc(socket_handle_size());
			let multiplex = alloc(socket_multiplex_handle_size());
			// open sockets an accept the inbound socket
			let port = socket_listen(server, addr.as_ptr(), 0, 10);
			assert_eq!(socket_connect(client, addr.as_ptr(), port), 0);
			assert_eq!(socket_accept(server, accepted), 0);

			assert_eq!(socket_multiplex_init(multiplex), 0);
			// register read
			assert_eq!(socket_multiplex_register(multiplex, accepted, 1), 0);
			let buf: [u8; 1] = [b'h'];
			let mut recv_buf = [0u8; 1];
			assert_eq!(socket_send(client, buf.as_ptr(), 1), 1);
			assert_eq!(socket_multiplex_wait(multiplex, events, 10), 1);
			// it's the first event so no offset
			assert!(socket_event_is_read(events));
			assert!(!socket_event_is_write(events));

			// get the readable handle
			let readablehandle = alloc(socket_handle_size());
			socket_event_handle(readablehandle, events);
			assert_eq!(recv_buf, [0u8; 1]);
			assert_eq!(socket_recv(readablehandle, recv_buf.as_mut_ptr(), 1), 1);
			assert_eq!(recv_buf, buf);

			// shutdown the socket
			socket_shutdown(accepted);
			assert_eq!(socket_multiplex_wait(multiplex, events, 10), 1);
			socket_event_handle(readablehandle, events);
			// this is a close so 0 bytes available
			assert_eq!(socket_recv(readablehandle, recv_buf.as_mut_ptr(), 1), 0);

			// close all three sockets
			assert_eq!(socket_close(server), 0);
			assert_eq!(socket_close(client), 0);
			assert_eq!(socket_close(accepted), 0);

			// release memory
			release(server);
			release(client);
			release(accepted);
			release(readablehandle);
			release(events);
			release(multiplex);
		}
	}

	#[test]
	fn test_backtrace() {
		unsafe {
			let x = backtrace_full();
			let mut itt = x;
			let mut count = 0;
			loop {
				if *itt == b'\0' {
					break;
				}
				itt = itt.wrapping_add(1);
				count += 1;
			}
			//println!("count={}", count);
			let v = from_utf8_unchecked(from_raw_parts(x, count));
			//println!("v={}", v);
		}
	}
}
