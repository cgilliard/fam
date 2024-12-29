use core::mem::size_of;
use core::ptr::{copy_nonoverlapping, drop_in_place, null_mut};
use core::slice::from_raw_parts;
//use core::str::from_utf8_unchecked;
use prelude::*;
use sys::*;

const REG_READ_FLAG: i32 = 0x1;
//const REG_WRITE_FLAG: i32 = 0x1 << 1;

const EAGAIN: i32 = -11;

const BAD_REQUEST: &str = "HTTP/1.1 400 Bad Request\r\n\
Content-Type: text/plain\r\n\
Connection: close\r\n\r\n";
const SWITCH_PROTOCOL: &str = "HTTP/1.1 101 Switching Protocols\r\n\
Upgrade: websocket\r\n\
Connection: Upgrade\r\n\
Sec-WebSocket-Accept: ";

const GET_PREFIX: &[u8] = "GET /".as_bytes();
const SEC_KEY_PREFIX: &[u8] = "Sec-WebSocket-Key: ".as_bytes();
const SEC_WEBSOCKET_PROTOCOLS: &[u8] = "Sec-WebSocket-Protocol: ".as_bytes();
const SEC_WEBSOCKET_VERSION: &[u8] = "Sec-WebSocket-Version: ".as_bytes();

pub struct WsConfig {
	addr: [u8; 4],
	port: u16,
	backlog: i32,
	threads: usize,
	max_events: i32,
}

impl Default for WsConfig {
	fn default() -> Self {
		Self {
			addr: [127, 0, 0, 1],
			port: 0, // randomly selected port
			backlog: 10,
			threads: 4,
			max_events: 30,
		}
	}
}

pub struct WsMessage {}

pub struct WsHandle {}

pub struct WsServer {
	config: WsConfig,
	port: u16,
	handle: *mut u8,
	wakeup: [u8; 8],
	stop: Rc<u64>,
	jhs: Rc<Vec<JoinHandle>>,
	lock: LockBox,
}

enum ConnectionState {
	NeedHandshake,
	HandshakeComplete,
}

#[allow(dead_code)]
struct ConnectionInner {
	read: Vec<u8>,
	write: Vec<u8>,
	id: u64,
	handle: [u8; 4],
	lock: Lock,
	state: ConnectionState,
}

impl Drop for ConnectionInner {
	fn drop(&mut self) {}
}

struct Connection {
	inner: Rc<ConnectionInner>,
}

struct Handle {
	inner: Rc<ConnectionInner>,
}

impl PartialEq for Connection {
	fn eq(&self, other: &Connection) -> bool {
		self.inner.id == other.inner.id
	}
}

impl Hash for Connection {
	fn hash(&self) -> usize {
		let slice =
			unsafe { from_raw_parts(&self.inner.id as *const u64 as *const u8, size_of::<u64>()) };
		murmur3_32_of_slice(slice, get_murmur_seed()) as usize
	}
}

impl PartialEq for Handle {
	fn eq(&self, other: &Handle) -> bool {
		self.inner.handle == other.inner.handle
	}
}

impl Hash for Handle {
	fn hash(&self) -> usize {
		murmur3_32_of_slice(&self.inner.handle, get_murmur_seed()) as usize
	}
}

#[allow(dead_code)]
struct WsContext {
	connections: Hashtable<Connection>,
	handles: Hashtable<Handle>,
	itt: Rc<u64>,
	id: Rc<u64>,
	stop: Rc<u64>,
	tid: u64,
	multiplex: *mut u8,
	events: *mut u8,
	handle: *mut u8,
	fhandle: Handle,
	wakeup: [u8; 8],
	jhs: Rc<Vec<JoinHandle>>,
}

impl WsContext {
	fn new(
		itt: Rc<u64>,
		id: Rc<u64>,
		tid: u64,
		config: &WsConfig,
		handle: *mut u8,
		mut wakeup: [u8; 8],
		stop: Rc<u64>,
		jhs: Rc<Vec<JoinHandle>>,
	) -> Result<Self, Error> {
		let connections = match Hashtable::new(1024) {
			Ok(connections) => connections,
			Err(e) => return Err(e),
		};
		let handles = match Hashtable::new(1024) {
			Ok(handles) => handles,
			Err(e) => return Err(e),
		};
		let multiplex = safe_alloc(safe_socket_multiplex_handle_size());
		if safe_socket_multiplex_init(multiplex) < 0 {
			safe_release(multiplex);
			return Err(ErrorKind::CreateFileDescriptor.into());
		}

		let wakeup_ptr = &mut wakeup as *mut u8;
		safe_socket_multiplex_register(multiplex, wakeup_ptr, REG_READ_FLAG);

		let events = safe_alloc(safe_socket_event_size() * config.max_events as usize);

		if safe_socket_multiplex_register(multiplex, handle, REG_READ_FLAG) < 0 {
			safe_release(multiplex);
			safe_release(events);
			return Err(ErrorKind::MultiplexRegister.into());
		}

		let fhandle = Handle {
			inner: Rc::new(ConnectionInner {
				handle: [0u8; 4],
				id: 0,
				lock: Lock::new(),
				read: Vec::new(),
				write: Vec::new(),
				state: ConnectionState::NeedHandshake,
			})
			.unwrap(),
		};

		Ok(Self {
			connections,
			handles,
			itt,
			id,
			tid,
			multiplex,
			events,
			handle,
			fhandle,
			wakeup,
			stop,
			jhs,
		})
	}
}

impl WsServer {
	pub fn new(config: WsConfig) -> Result<Self, Error> {
		let port = config.port;
		let handle = null_mut();
		let wakeup = [0u8; 8];

		let stop = match Rc::new(0) {
			Ok(stop) => stop,
			Err(e) => return Err(e),
		};

		let jhs = match Rc::new(Vec::new()) {
			Ok(jhs) => jhs,
			Err(e) => return Err(e),
		};

		let lock = match lock_box!() {
			Ok(lock) => lock,
			Err(e) => return Err(e),
		};

		Ok(Self {
			config,
			port,
			handle,
			wakeup,
			stop,
			jhs,
			lock,
		})
	}

	pub fn register_handler(
		&mut self,
		_handler: Box<dyn FnMut(WsMessage, WsHandle) -> Result<(), Error>>,
	) -> Result<(), Error> {
		Ok(())
	}

	pub fn start(&mut self) -> Result<(), Error> {
		match self.bind_socket() {
			Ok(_) => {}
			Err(e) => return Err(e),
		};

		let stop = match self.stop.clone() {
			Ok(stop) => stop,
			Err(e) => return Err(e),
		};

		let jhs = match self.jhs.clone() {
			Ok(jhs) => jhs,
			Err(e) => return Err(e),
		};

		let lock = match self.lock.clone() {
			Ok(lock) => lock,
			Err(e) => return Err(e),
		};

		let wakeup_ptr = &mut self.wakeup as *mut u8;
		safe_pipe(wakeup_ptr);

		match Self::start_threads(&self.config, self.handle, self.wakeup, stop, jhs, lock) {
			Ok(_) => {}
			Err(e) => return Err(e),
		}

		Ok(())
	}

	pub fn stop(&mut self) -> Result<(), Error> {
		let wakeup = &mut self.wakeup as *mut u8;
		let wakeup = unsafe { wakeup.add(4) };
		astore!(&mut *self.stop, 1);
		safe_socket_send(wakeup, "1".as_ptr(), 1);

		{
			let _l = self.lock.write();
			for i in 0..self.jhs.len() {
				let _ = self.jhs[i].join();
			}
		}

		Ok(())
	}

	fn bind_socket(&mut self) -> Result<(), Error> {
		self.handle = safe_alloc(safe_socket_handle_size());
		self.port = safe_socket_listen(
			self.handle,
			self.config.addr.as_ptr(),
			self.config.port,
			self.config.backlog,
		) as u16;

		Ok(())
	}

	fn start_threads(
		config: &WsConfig,
		handle: *mut u8,
		wakeup: [u8; 8],
		stop: Rc<u64>,
		jhs: Rc<Vec<JoinHandle>>,
		lock: LockBox,
	) -> Result<(), Error> {
		let itt = match Rc::new(0) {
			Ok(itt) => itt,
			Err(e) => return Err(e),
		};
		let id = match Rc::new(0) {
			Ok(id) => id,
			Err(e) => return Err(e),
		};

		for tid in 0..config.threads {
			let jhs = match jhs.clone() {
				Ok(jhs) => jhs,
				Err(e) => return Err(e),
			};
			let ctx = match itt.clone() {
				Ok(itt) => match id.clone() {
					Ok(id) => match stop.clone() {
						Ok(stop) => {
							match WsContext::new(
								itt, id, tid as u64, config, handle, wakeup, stop, jhs,
							) {
								Ok(ctx) => ctx,
								Err(e) => return Err(e),
							}
						}
						Err(e) => return Err(e),
					},
					Err(e) => return Err(e),
				},
				Err(e) => return Err(e),
			};

			let _l = lock.write();
			match Self::thread_init(config, ctx) {
				Ok(_) => {}
				Err(e) => return Err(e),
			}
		}
		Ok(())
	}

	fn proc_read(ctx: &mut WsContext, ehandle: *mut u8) {
		unsafe {
			copy_nonoverlapping(
				ehandle,
				ctx.fhandle.inner.handle.as_mut_ptr(),
				ctx.fhandle.inner.handle.len(),
			);
		}
		let mut handle = ctx.handles.find(&ctx.fhandle).unwrap();

		let rlen = handle.inner.read.len();
		handle.inner.read.resize(rlen + 512).unwrap();
		let buf = &handle.inner.read[rlen..rlen + 512];
		let len = safe_socket_recv(ehandle, buf.as_ptr(), 512);

		if len == 0 {
			safe_socket_close(ehandle);
			let to_drop = ctx.handles.remove(&ctx.fhandle).unwrap();
			unsafe {
				drop_in_place(to_drop.raw());
			}
			return;
		}

		handle.inner.read.resize(len as usize + rlen).unwrap();
		Self::proc_messages(&mut handle, ehandle);
	}

	fn bad_request(ehandle: *mut u8) {
		safe_socket_send(ehandle, BAD_REQUEST.as_ptr(), BAD_REQUEST.len());
		safe_socket_shutdown(ehandle);
	}

	pub fn handle_websocket_handshake(sec_key: &[u8]) -> [u8; 28] {
		let magic_string = b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
		let mut sha1_result: [u8; 20] = [0; 20];
		let mut combined: [u8; 60] = [0; 60];

		unsafe {
			copy_nonoverlapping(sec_key.as_ptr(), combined.as_mut_ptr(), sec_key.len());

			copy_nonoverlapping(
				magic_string.as_ptr(),
				combined.as_mut_ptr().add(sec_key.len()),
				magic_string.len(),
			);
			SHA1(combined.as_ptr(), combined.len(), sha1_result.as_mut_ptr());

			let mut accept_key: [u8; 28] = [0; 28];
			Base64encode(
				accept_key.as_mut_ptr(),
				sha1_result.as_mut_ptr(),
				sha1_result.len(),
			);

			accept_key
		}
	}

	fn switch_protocol(ehandle: *mut u8, accept_key: &[u8; 28]) {
		safe_socket_send(ehandle, SWITCH_PROTOCOL.as_ptr(), SWITCH_PROTOCOL.len());
		safe_socket_send(ehandle, accept_key.as_ptr(), accept_key.len());
		safe_socket_send(ehandle, "\r\n\r\n".as_ptr(), 4);
	}

	fn proc_messages(handle: &mut Handle, ehandle: *mut u8) {
		match handle.inner.state {
			ConnectionState::NeedHandshake => {
				let len = handle.inner.read.len();
				let rvec = &handle.inner.read;
				let mut uri_end = 0;
				if len >= 5 && &rvec[0..5] == GET_PREFIX {
					for i in 5..len {
						if rvec[i] == b' ' || rvec[i] == b'\r' || rvec[i] == b'\n' {
							uri_end = i;
							break;
						}
					}
					if uri_end == 0 {
						Self::bad_request(ehandle);
						return;
					}

					let mut sec_key: &[u8] = &[];
					let mut version: &[u8] = &[];
					let mut protocols: &[u8] = &[];

					for i in uri_end..len {
						if rvec[i] == b'\n'
							&& rvec[i - 1] == b'\r'
							&& rvec[i - 2] == b'\n'
							&& rvec[i - 3] == b'\r'
						{
							if version != &[b'1', b'3']
								|| protocols != &[b'c', b'h', b'a', b't']
								|| sec_key.len() != 24
							{
								Self::bad_request(ehandle);
								return;
							}

							let accept_key = Self::handle_websocket_handshake(sec_key);
							Self::switch_protocol(ehandle, &accept_key);
							handle.inner.read.clear();
							handle.inner.state = ConnectionState::HandshakeComplete;
							break;
						} else if rvec[i] == b'\n'
							&& len > i + 1 + SEC_KEY_PREFIX.len()
							&& &rvec[i + 1..i + 1 + SEC_KEY_PREFIX.len()] == SEC_KEY_PREFIX
						{
							for j in i + 1 + SEC_KEY_PREFIX.len()..len {
								if rvec[j] == b'\r' || rvec[j] == b'\n' {
									sec_key = &rvec[i + 1 + SEC_KEY_PREFIX.len()..j];
									break;
								}
							}
						} else if rvec[i] == b'\n'
							&& len > i + 1 + SEC_WEBSOCKET_VERSION.len()
							&& &rvec[i + 1..i + 1 + SEC_WEBSOCKET_VERSION.len()]
								== SEC_WEBSOCKET_VERSION
						{
							for j in i + 1 + SEC_WEBSOCKET_VERSION.len()..len {
								if rvec[j] == b'\r' || rvec[j] == b'\n' {
									version = &rvec[i + 1 + SEC_WEBSOCKET_VERSION.len()..j];
									break;
								}
							}
						} else if rvec[i] == b'\n'
							&& len > i + 1 + SEC_WEBSOCKET_PROTOCOLS.len()
							&& &rvec[i + 1..i + 1 + SEC_WEBSOCKET_PROTOCOLS.len()]
								== SEC_WEBSOCKET_PROTOCOLS
						{
							for j in i + 1 + SEC_WEBSOCKET_PROTOCOLS.len()..len {
								if rvec[j] == b'\r' || rvec[j] == b'\n' {
									protocols = &rvec[i + 1 + SEC_WEBSOCKET_PROTOCOLS.len()..j];
									break;
								}
							}
						}
					}
				} else {
					Self::bad_request(ehandle);
					return;
				}
			}
			_ => {}
		}
	}

	fn proc_accept(ctx: &mut WsContext, ehandle: *mut u8) {
		loop {
			let mut handle = [0u8; 4];
			let nhandle: *mut u8 = &mut handle as *mut u8;
			let res = safe_socket_accept(ehandle, nhandle);
			if res < 0 {
				if res == EAGAIN {
					break;
				} else {
					println!("WARN: Error accepting socket code: {}", res);
					continue;
				}
			}
			if safe_socket_multiplex_register(ctx.multiplex, nhandle, REG_READ_FLAG) < 0 {
				println!("WARN: could not register accepted connection!");
				safe_socket_close(nhandle);
			}

			let conn = Handle {
				inner: Rc::new(ConnectionInner {
					handle,
					id: aadd!(&mut *ctx.id, 1),
					lock: Lock::new(),
					read: Vec::new(),
					write: Vec::new(),
					state: ConnectionState::NeedHandshake,
				})
				.unwrap(),
			};
			let conn = Ptr::alloc(Node::new(conn)).unwrap();
			ctx.handles.insert(conn);
		}
	}

	fn thread_init(config: &WsConfig, mut ctx: WsContext) -> Result<(), Error> {
		let mut jhs = match ctx.jhs.clone() {
			Ok(jhs) => jhs,
			Err(e) => return Err(e),
		};
		let s = spawnj(move || {
			let mut ehandle = [0u8; 4];
			let ehandle: *mut u8 = &mut ehandle as *mut u8;
			let wakeup: *mut u8 = &mut ctx.wakeup as *mut u8;
			let mut stop = false;
			loop {
				let count =
					safe_socket_multiplex_wait(ctx.multiplex, ctx.events, config.max_events);
				for i in 0..count {
					safe_socket_event_handle(ehandle, unsafe {
						ctx.events
							.add(i as usize * safe_socket_event_size() as usize)
					});

					if safe_socket_as_i32(ehandle) == safe_socket_as_i32(ctx.handle) {
						// since we are edge triggered, no other events
						// can fire until we accept the connections, so
						// we know this can only happen in each thread once
						let cur = aload!(&*ctx.itt);
						let rem = if config.threads != 0 {
							cur as usize % config.threads
						} else {
							1
						};
						if config.threads != 0 && rem == ctx.tid as usize {
							Self::proc_accept(&mut ctx, ehandle);
							aadd!(&mut *ctx.itt, 1);
						}
					} else if safe_socket_as_i32(ehandle) == safe_socket_as_i32(wakeup) {
						if aload!(&*ctx.stop) != 0 {
							stop = true;
							break;
						}
					} else {
						Self::proc_read(&mut ctx, ehandle);
					}
				}
				if stop {
					break;
				}
			}
		});

		match s {
			Ok(jh) => match jhs.push(jh) {
				Ok(_) => Ok(()),
				Err(e) => Err(e),
			},
			Err(e) => Err(e),
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use core::str::from_utf8_unchecked;

	#[test]
	fn test_ws1() {
		let initial = unsafe { getalloccount() };
		{
			let config = WsConfig {
				port: 9999,
				..Default::default()
			};
			let mut ws = WsServer::new(config).unwrap();
			ws.start().unwrap();
			let handle = safe_alloc(safe_socket_handle_size());
			let addr = [127u8, 0u8, 0u8, 1u8];
			safe_socket_connect(handle, &addr as *const u8, 9999);
			safe_socket_send(handle, "POST /\r\n".as_ptr(), 8);
			let mut buf = [0u8; 512];
			let x = safe_socket_recv(handle, &mut buf as *mut u8, 512);
			let start = "HTTP/1.1 400";
			assert!(x > start.len() as i64);
			assert_eq!(unsafe { from_utf8_unchecked(&buf[0..start.len()]) }, start);
			ws.stop().unwrap();
			safe_release(handle);
		}
		//assert_eq!(initial, unsafe { getalloccount() });
	}
}
