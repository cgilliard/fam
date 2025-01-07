#![allow(dead_code)]
#![allow(unused_variables)]

use core::ptr::{copy_nonoverlapping, null_mut};
use prelude::*;
use sys::*;

const MAGIC_STRING: &[u8; 36] = b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
const BAD_REQUEST: &str = "HTTP/1.1 400 Bad Request\r\n\
Content-Type: text/plain\r\n\
Connection: close\r\n\r\n";
const SWITCH_PROTOCOL: &str = "HTTP/1.1 101 Switching Protocols\r\n\
Upgrade: websocket\r\n\
Connection: Upgrade\r\n\
Sec-WebSocket-Accept: ";
const SWITCHING_PROTOCOL_PREFIX: &str = "HTTP/1.1 101 Switching Protocols\r\n";
const CONNECT_MESSAGE_PREFIX: &str = "GET / HTTP/1.1\r\n\
Sec-WebSocket-Key: ";

const GET_PREFIX: &[u8] = "GET /".as_bytes();
const SEC_KEY_PREFIX: &[u8] = "Sec-WebSocket-Key: ".as_bytes();

const EAGAIN: i32 = -11;
const REG_READ_FLAG: i32 = 0x1;
const REG_WRITE_FLAG: i32 = 0x2;

#[derive(PartialEq)]
enum ConnectionState {
	NeedHandshake,
	HandshakeComplete,
	Closed,
}

#[derive(PartialEq, Clone, Copy)]
enum ConnectionType {
	Server,
	ServerConnection,
	ClientConnection,
}

pub struct WsConfig {
	threads: u64,
	max_events: i32,
	debug_pending: bool,
}

enum ConnectionMessage {
	Read(Box<Connection>),
	Write(Ptr<Connection>),
}

struct ConnectionInner {
	next: Ptr<Connection>,
	prev: Ptr<Connection>,
	connptr: Ptr<Connection>,
	ctype: ConnectionType,
	cstate: ConnectionState,
	rbuf: Vec<u8>,
	wbuf: Vec<u8>,
	handle: [u8; 4],
	lock: Lock,
	send: Sender<ConnectionMessage>,
	debug_pending: bool,
	wakeup: [u8; 8],
}

struct Connection {
	inner: Rc<ConnectionInner>,
}

pub struct WsRequest<'a> {
	msg: &'a [u8],
	fin: bool,
	op: u8,
}

enum MessageType {
	Text,
	Binary,
}

pub struct WsResponse {
	conn: Connection,
}

pub struct WsServerConfig {
	addr: [u8; 4],
	port: u16,
	backlog: i32,
}

pub struct WsClientConfig {
	addr: [u8; 4],
	port: u16,
}

struct WorkerState {
	head: *mut Connection,
	wakeup: [u8; 8],
	mplex: [u8; 4],
	recv: Receiver<ConnectionMessage>,
	send: Sender<ConnectionMessage>,
	comp_recv: Receiver<()>,
	comp_send: Sender<()>,
}

struct State {
	wstate: Vec<WorkerState>,
	runtime: Option<Runtime<()>>,
	handler: Option<Box<dyn FnMut(WsRequest, WsResponse) -> Result<(), Error>>>,
	config: WsConfig,
	itt: u64,
	lock: LockBox,
	halt: bool,
}

pub struct WsContext {
	state: Rc<State>,
	tid: usize,
	events: *mut u8,
}

pub struct WebSocket {
	state: Rc<State>,
}

impl Clone for WsResponse {
	fn clone(&self) -> Result<Self, Error> {
		Ok(Self {
			conn: self.conn.clone().unwrap(),
		})
	}
}

impl WsResponse {
	pub fn send(&mut self, msg: &str) -> Result<(), Error> {
		self.send_impl(MessageType::Text, msg.as_bytes())
	}

	pub fn sendb(&mut self, msg: &[u8]) -> Result<(), Error> {
		self.send_impl(MessageType::Binary, msg)
	}

	pub fn close(&self, status: u16) {
		self.conn.close(status);
	}

	fn send_impl(&mut self, mtype: MessageType, bytes: &[u8]) -> Result<(), Error> {
		let _l = self.conn.inner.lock.write();
		let b1 = match mtype {
			MessageType::Text => 0x81,
			MessageType::Binary => 0x82,
		};

		if bytes.len() <= 125 {
			match self.conn.writeb(&[b1, bytes.len() as u8]) {
				Ok(_) => {}
				Err(e) => {
					self.conn.close(1011);
					return Err(e);
				}
			}
		} else if bytes.len() <= 65535 {
			match self.conn.writeb(&[b1, 126]) {
				Ok(_) => {}
				Err(e) => {
					self.conn.close(1011);
					return Err(e);
				}
			}
			let mut len = [0u8; 2];
			to_be_bytes_u16(bytes.len() as u16, &mut len);
			match self.conn.writeb(&len) {
				Ok(_) => {}
				Err(e) => {
					self.conn.close(1011);
					return Err(e);
				}
			}
		} else {
			match self.conn.writeb(&[b1, 127]) {
				Ok(_) => {}
				Err(e) => {
					self.conn.close(1011);
					return Err(e);
				}
			}
			let mut len = [0u8; 8];
			to_be_bytes_u64(bytes.len() as u64, &mut len);
			match self.conn.writeb(&len) {
				Ok(_) => {}
				Err(e) => {
					self.conn.close(1011);
					return Err(e);
				}
			}
		}

		match self.conn.writeb(bytes) {
			Ok(_) => {}
			Err(e) => {
				self.conn.close(1011);
				return Err(e);
			}
		}
		Ok(())
	}
}

impl WsRequest<'_> {
	pub fn msg(&self) -> &[u8] {
		self.msg
	}
}

impl Default for WsConfig {
	fn default() -> Self {
		Self {
			threads: 4,
			max_events: 32,
			debug_pending: false,
		}
	}
}

impl Clone for Connection {
	fn clone(&self) -> Result<Self, Error> {
		Ok(Self {
			inner: self.inner.clone().unwrap(),
		})
	}
}

impl Connection {
	fn new(
		ctype: ConnectionType,
		handle: [u8; 4],
		send: Sender<ConnectionMessage>,
		debug_pending: bool,
		wakeup: [u8; 8],
	) -> Result<Self, Error> {
		let mut rbuf = Vec::new();
		rbuf.set_min(0);
		match Rc::new(ConnectionInner {
			next: Ptr::null(),
			prev: Ptr::null(),
			connptr: Ptr::null(),
			ctype,
			rbuf,
			wbuf: Vec::new(),
			handle,
			lock: lock!(),
			cstate: ConnectionState::NeedHandshake,
			send,
			debug_pending,
			wakeup,
		}) {
			Ok(inner) => Ok(Self { inner }),
			Err(e) => Err(e),
		}
	}

	fn writeb(&self, msg: &[u8]) -> Result<(), Error> {
		let mut inner = self.inner.clone().unwrap();
		if self.inner.cstate == ConnectionState::Closed {
			return Err(err!(ConnectionClosed));
		}
		let mut res = if inner.wbuf.len() == 0 && !self.inner.debug_pending {
			safe_socket_send(&inner.handle as *const u8, msg.as_ptr(), msg.len())
		} else {
			0
		};
		if res == EAGAIN.into() || (res >= 0 && (res as usize) < msg.len()) {
			if res < 0 {
				res = 0;
			}
			unsafe {
				match inner
					.wbuf
					.append_ptr(msg.as_ptr().add(res as usize), msg.len() - (res as usize))
				{
					Ok(_) => {}
					Err(_e) => {
						// could not allocate space to append data to buffer. Close socket.
						println!(
							"WARN: Could not allocate space to write buffer. Dropping connection!"
						);
						let _ = self.close(1011);
						return Err(err!(IO));
					}
				}
			}

			match self
				.inner
				.send
				.send(ConnectionMessage::Write(self.inner.connptr))
			{
				Ok(_) => {}
				Err(e) => return Err(e),
			}

			safe_socket_send(
				unsafe { (&self.inner.wakeup as *const u8).add(4) },
				&b'0',
				1,
			);
		} else if res < 0 {
			safe_socket_shutdown(&self.inner.handle as *const u8);
		}

		Ok(())
	}

	fn write(&self, msg: &str) -> Result<(), Error> {
		self.writeb(msg.as_bytes())
	}

	pub fn close(&self, v: u16) {
		let mut status_code = [0u8; 2];
		to_be_bytes_u16(v, &mut status_code);
		let _ = self.writeb(&[0x88, 0]);
		let _ = self.writeb(&[0x88, 2]);
		let _ = self.writeb(&status_code);
		safe_socket_shutdown(&self.inner.handle as *const u8);
	}
}

impl WorkerState {
	fn new(wakeup: [u8; 8], mplex: [u8; 4]) -> Result<Self, Error> {
		let (send, recv) = match channel() {
			Ok((send, recv)) => (send, recv),
			Err(e) => return Err(e),
		};
		let (comp_send, comp_recv) = match channel() {
			Ok((send, recv)) => (send, recv),
			Err(e) => return Err(e),
		};
		Ok(Self {
			mplex,
			wakeup,
			head: null_mut(),
			send,
			recv,
			comp_send,
			comp_recv,
		})
	}
}

impl State {
	fn new(config: WsConfig) -> Result<Self, Error> {
		let lock = match lock_box!() {
			Ok(lock) => lock,
			Err(e) => return Err(e),
		};

		Ok(Self {
			runtime: None,
			wstate: Vec::new(),
			config,
			handler: None,
			itt: 0,
			lock,
			halt: false,
		})
	}
}

impl WebSocket {
	pub fn new(config: WsConfig) -> Result<Self, Error> {
		let state = match State::new(config) {
			Ok(state) => state,
			Err(e) => return Err(e),
		};
		Ok(Self {
			state: Rc::new(state).unwrap(),
		})
	}

	pub fn add_client(&mut self, config: WsClientConfig) -> Result<WsResponse, Error> {
		let mut client = [0u8; 4];
		let client_ptr = &mut client as *mut u8;
		if safe_socket_connect(client_ptr, config.addr.as_ptr(), config.port as i32) < 0 {
			return Err(err!(Connect));
		}
		let threads = self.state.config.threads;
		let itt = if threads > 0 {
			(aadd!(&mut self.state.itt, 1) % threads) as usize
		} else {
			1
		};
		let mplex = self.state.wstate[itt].mplex;
		let conn = match Connection::new(
			ConnectionType::ClientConnection,
			client,
			self.state.wstate[itt].send.clone().unwrap(),
			self.state.config.debug_pending,
			self.state.wstate[itt].wakeup,
		) {
			Ok(conn) => conn,
			Err(e) => {
				safe_socket_close(client_ptr);
				return Err(e);
			}
		};

		let mut boxed_conn = match Box::new(conn.clone().unwrap()) {
			Ok(conn) => conn,
			Err(e) => {
				safe_socket_close(client_ptr);
				return Err(e);
			}
		};
		boxed_conn.leak();
		// note: we simplify here and return an error if the full message is not sent.
		// these are short and should generally succeed. Re-try logic can be used by
		// caller.
		if safe_socket_send(
			client_ptr,
			CONNECT_MESSAGE_PREFIX.as_ptr(),
			CONNECT_MESSAGE_PREFIX.len(),
		) < CONNECT_MESSAGE_PREFIX.len() as i64
		{
			safe_socket_close(client_ptr);
			return Err(err!(IO));
		}
		let mut accept_key: [u8; 24] = [0; 24];
		let mut rand_bytes: [u8; 16] = [0; 16];
		// TODO: switch to secure psrng
		safe_rand_bytes(&mut rand_bytes as *mut u8, rand_bytes.len());
		unsafe {
			Base64encode(
				accept_key.as_mut_ptr(),
				rand_bytes.as_mut_ptr(),
				rand_bytes.len(),
			);
		}

		if safe_socket_send(client_ptr, accept_key.as_ptr(), accept_key.len())
			< accept_key.len() as i64
		{
			safe_socket_close(client_ptr);
			return Err(err!(IO));
		}
		if safe_socket_send(client_ptr, "\r\n\r\n".as_ptr(), 4) < 4 {
			safe_socket_close(client_ptr);
			return Err(err!(IO));
		}

		match self.state.wstate[itt]
			.send
			.send(ConnectionMessage::Read(boxed_conn))
		{
			Ok(_) => {}
			Err(e) => return Err(e),
		}

		if safe_socket_send(
			unsafe { (&self.state.wstate[itt].wakeup as *const u8).add(4) },
			&b'0',
			1,
		) < 1
		{
			safe_socket_close(client_ptr);
			return Err(err!(IO));
		}
		self.state.wstate[itt].comp_recv.recv();

		Ok(WsResponse { conn })
	}

	pub fn add_server(&mut self, config: WsServerConfig) -> Result<u16, Error> {
		let mut server = [0u8; 4];
		let server_ptr = &mut server as *mut u8;
		let port = safe_socket_listen(
			server_ptr,
			config.addr.as_ptr(),
			config.port,
			config.backlog,
		);
		if port < 0 {
			return Err(err!(Bind));
		}

		let mut i = 0;
		for wstate in &self.state.wstate {
			let connection = match Connection::new(
				ConnectionType::Server,
				server,
				self.state.wstate[i].send.clone().unwrap(),
				self.state.config.debug_pending,
				self.state.wstate[i].wakeup,
			) {
				Ok(connection) => connection,
				Err(e) => return Err(e),
			};

			let mut connection = match Box::new(connection) {
				Ok(connection) => connection,
				Err(e) => return Err(e),
			};
			connection.leak();

			match wstate.send.send(ConnectionMessage::Read(connection)) {
				Ok(_) => {}
				Err(e) => return Err(e),
			}
			if safe_socket_send(unsafe { (&wstate.wakeup as *const u8).add(4) }, &b'0', 1) < 1 {
				return Err(err!(WsStop));
			}

			wstate.comp_recv.recv();
			i += 1;
		}

		Ok(port as u16)
	}

	pub fn stop(&mut self) -> Result<(), Error> {
		let lock = self.state.lock.clone().unwrap();
		{
			let _l = lock.write();
			self.state.halt = true;
		}
		match self.wakeup_threads() {
			Ok(_) => {}
			Err(e) => return Err(e),
		}
		match &mut self.state.runtime {
			Some(ref mut rt) => rt.stop(),
			None => Ok(()),
		}
	}

	fn wakeup_threads(&self) -> Result<(), Error> {
		for wstate in &self.state.wstate {
			if safe_socket_send(unsafe { (&wstate.wakeup as *const u8).add(4) }, &b'0', 1) < 1 {
				println!("WARN: could not wakeup fd {}", unsafe {
					socket_fd((&wstate.wakeup as *const u8).add(4) as *const u8)
				});
				return Err(err!(WsStop));
			}
		}
		Ok(())
	}

	pub fn register_handler(
		&mut self,
		handler: Box<dyn FnMut(WsRequest, WsResponse) -> Result<(), Error>>,
	) {
		self.state.handler = Some(handler);
	}

	pub fn start(&mut self) -> Result<(), Error> {
		let runtime_config = RuntimeConfig {
			max_threads: self.state.config.threads,
			min_threads: self.state.config.threads,
		};

		let mut runtime: Runtime<()> = match Runtime::new(runtime_config) {
			Ok(runtime) => runtime,
			Err(e) => return Err(e),
		};
		match runtime.start() {
			Ok(_) => {}
			Err(e) => return Err(e),
		}

		for tid in 0..self.state.config.threads as usize {
			let mut state = self.state.clone().unwrap();
			let mut mplex = [0u8; 4];

			if safe_socket_multiplex_init(&mut mplex as *mut u8) < 0 {
				return Err(err!(CreateFileDescriptor));
			}

			let mut wakeup = [0u8; 8];
			if safe_pipe(&mut wakeup as *mut u8) < 0 {
				return Err(err!(Pipe));
			}

			let wstate = match WorkerState::new(wakeup, mplex) {
				Ok(wstate) => wstate,
				Err(e) => return Err(e),
			};

			match state.wstate.push(wstate) {
				Ok(_) => {}
				Err(e) => return Err(e),
			}

			if safe_socket_multiplex_register(
				&mplex as *const u8,
				&wakeup as *const u8,
				REG_READ_FLAG,
				null_mut(),
			) < 0
			{
				return Err(err!(MultiplexRegister));
			}
			let events =
				safe_alloc(safe_socket_event_size() * self.state.config.max_events as usize)
					as *mut u8;

			let mut ctx = WsContext { state, tid, events };

			let _ = runtime.execute(move || match Self::event_loop(&mut ctx) {
				Ok(_) => {}
				Err(e) => println!("FATAL: unexpected error in event_loop: {}", e),
			});
		}

		self.state.runtime = Some(runtime);

		Ok(())
	}

	fn remove_from_list(ctx: &mut WsContext, conn: &mut Box<Connection>) {
		if !conn.inner.prev.is_null() {
			conn.inner.prev.inner.next = conn.inner.next;
		} else {
			// update head
			ctx.state.wstate[ctx.tid].head = conn.inner.next.raw();
		}
		if !conn.inner.next.is_null() {
			conn.inner.next.inner.prev = conn.inner.prev;
		}
	}

	fn update_head(ctx: &mut WsContext, conn: &mut Box<Connection>) {
		let mut state_clone1 = ctx.state.clone().unwrap();
		let mut state_clone2 = ctx.state.clone().unwrap();
		conn.inner.next = Ptr::new(ctx.state.wstate[ctx.tid].head);
		conn.inner.prev = Ptr::null();
		if !ctx.state.wstate[ctx.tid].head.is_null() {
			unsafe {
				(*state_clone1.wstate[ctx.tid].head).inner.prev = Ptr::new(conn.as_ptr().raw());
			}
		}
		state_clone2.wstate[ctx.tid].head = conn.as_ptr().raw();
	}

	fn proc_wakeup(ctx: &mut WsContext) {
		let mplex = &ctx.state.wstate[ctx.tid].mplex as *const u8;
		while ctx.state.wstate[ctx.tid].recv.pending() {
			match ctx.state.wstate[ctx.tid].recv.recv() {
				ConnectionMessage::Read(mut conn) => {
					let _ = ctx.state.wstate[ctx.tid].comp_send.send(());
					conn.inner.connptr = conn.as_ptr();
					if safe_socket_multiplex_register(
						mplex as *const u8,
						&conn.inner.handle as *const u8,
						REG_READ_FLAG,
						conn.as_ptr().raw() as *const u8,
					) < 0
					{
						safe_socket_close(&conn.inner.handle as *const u8);
					} else {
						Self::update_head(ctx, &mut conn);
					}
				}
				ConnectionMessage::Write(conn) => {
					if safe_socket_multiplex_register(
						mplex as *const u8,
						&conn.inner.handle as *const u8,
						REG_READ_FLAG | REG_WRITE_FLAG,
						conn.raw() as *const u8,
					) < 0
					{
						safe_socket_close(&conn.inner.handle as *const u8);
					}
				}
			}
		}
	}

	fn handle_websocket_handshake(sec_key: &[u8]) -> [u8; 28] {
		let mut sha1_result: [u8; 20] = [0; 20];
		let mut combined: [u8; 60] = [0; 60];

		unsafe {
			copy_nonoverlapping(sec_key.as_ptr(), combined.as_mut_ptr(), sec_key.len());

			copy_nonoverlapping(
				MAGIC_STRING.as_ptr(),
				combined.as_mut_ptr().add(sec_key.len()),
				MAGIC_STRING.len(),
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

	fn switch_protocol(handle: &mut Box<Connection>, accept_key: &[u8; 28]) {
		match handle.write(SWITCH_PROTOCOL) {
			Ok(_) => {}
			Err(_e) => handle.close(1011),
		}
		match handle.writeb(accept_key) {
			Ok(_) => {}
			Err(_e) => handle.close(1011),
		}

		match handle.write("\r\n\r\n") {
			Ok(_) => {}
			Err(_e) => handle.close(1011),
		}
	}

	fn bad_request(handle: &mut Box<Connection>) {
		let _ = handle.write(BAD_REQUEST);
		safe_socket_shutdown(&mut handle.inner.handle as *const u8);
	}

	fn proc_hs_client(handle: &mut Box<Connection>) {
		let mut handle_clone = handle.clone().unwrap();
		let rvec = &handle.inner.rbuf;
		for i in 3..rvec.len() {
			if rvec[i] == b'\n'
				&& rvec[i - 1] == b'\r'
				&& rvec[i - 2] == b'\n'
				&& rvec[i - 3] == b'\r'
			{
				// end of response just check if this is a 101
				if i >= SWITCHING_PROTOCOL_PREFIX.len()
					&& &rvec[0..SWITCHING_PROTOCOL_PREFIX.len()]
						== SWITCHING_PROTOCOL_PREFIX.as_bytes()
				{
					handle_clone.inner.cstate = ConnectionState::HandshakeComplete;
					if rvec.len() == i + 1 {
						handle_clone.inner.rbuf.clear();
					} else {
						let _ = handle_clone.inner.rbuf.shift(i + 1);
					}
					break;
				}
			}
		}
	}

	fn proc_hs(handle: &mut Box<Connection>) {
		let mut handle_clone = handle.clone().unwrap();
		let len = handle.inner.rbuf.len();
		let rvec = &handle.inner.rbuf;
		let mut uri_end = 0;
		if len >= 5 && &rvec[0..5] == GET_PREFIX {
			for i in 5..len {
				if rvec[i] == b' ' || rvec[i] == b'?' || rvec[i] == b'\r' || rvec[i] == b'\n' {
					uri_end = i;
					break;
				}
			}
			if uri_end == 0 {
				Self::bad_request(handle);
				return;
			}

			let uri = &rvec[4..uri_end];
			for i in 0..uri.len() {
				if !((uri[i] >= b'a' && uri[i] <= b'z')
					|| (uri[i] >= b'A' && uri[i] <= b'Z')
					|| (uri[i] >= b'0' && uri[i] <= b'9')
					|| uri[i] == b'-'
					|| uri[i] == b'.'
					|| uri[i] == b'_'
					|| uri[i] == b'~'
					|| uri[i] == b'/')
				{
					Self::bad_request(handle);
					return;
				}
			}

			let mut sec_key: &[u8] = &[];

			for i in uri_end..len {
				if rvec[i] == b'\n'
					&& rvec[i - 1] == b'\r'
					&& rvec[i - 2] == b'\n'
					&& rvec[i - 3] == b'\r'
				{
					if sec_key == &[] || sec_key.len() > 24 {
						Self::bad_request(handle);
					} else {
						let accept_key = Self::handle_websocket_handshake(sec_key);
						Self::switch_protocol(handle, &accept_key);
						handle.inner.cstate = ConnectionState::HandshakeComplete;

						let rbuflen = handle_clone.inner.rbuf.len();
						if rbuflen == i + 1 {
							handle_clone.inner.rbuf.clear();
						} else {
							let _ = handle_clone.inner.rbuf.shift(i + 1);
						}
					}
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
				}
			}
		} else {
			Self::bad_request(handle);
			return;
		}
	}

	fn proc_hs_complete(handle: &mut Box<Connection>, ctx: &mut WsContext) {
		let conn = Connection {
			inner: handle.inner.clone().unwrap(),
		};

		let len = handle.inner.rbuf.len();

		// min length to try to process
		if len < 2 {
			return;
		}

		let rvec = &mut handle.inner.rbuf;
		let fin = rvec[0] & 0x80 != 0;

		// reserved bits not 0
		if rvec[0] & 0x70 != 0 {
			Self::close_cleanly(handle, 1002);
			return;
		}

		let op = rvec[0] & !0x80;
		let mask = rvec[1] & 0x80 != 0;

		// determine variable payload len
		let payload_len = rvec[1] & 0x7F;
		let (payload_len, mut offset) = if payload_len == 126 {
			if len < 4 {
				return;
			}
			((rvec[2] as usize) << 8 | rvec[3] as usize, 4)
		} else if payload_len == 127 {
			if len < 10 {
				return;
			}
			(
				(rvec[2] as usize) << 56
					| (rvec[3] as usize) << 48
					| (rvec[4] as usize) << 40
					| (rvec[5] as usize) << 32
					| (rvec[6] as usize) << 24
					| (rvec[7] as usize) << 16
					| (rvec[8] as usize) << 8
					| (rvec[9] as usize),
				10,
			)
		} else {
			(payload_len as usize, 2)
		};

		// if masking set we add 4 bytes for the masking key
		if mask {
			offset += 4;
			if offset + payload_len > len {
				return;
			}
			let masking_key = [
				rvec[offset - 4],
				rvec[offset - 3],
				rvec[offset - 2],
				rvec[offset - 1],
			];

			for i in 0..payload_len {
				if i % 4 < masking_key.len() && offset + i < rvec.len() {
					rvec[offset + i] ^= masking_key[i % 4];
				}
			}
		}

		if offset + payload_len > len {
			return;
		}
		let payload = &rvec[offset..payload_len + offset];

		let req = WsRequest {
			fin,
			op,
			msg: payload,
		};
		let resp = WsResponse { conn };
		match &mut ctx.state.handler {
			Some(handler) => match handler(req, resp) {
				Ok(_) => {}
				Err(e) => println!("WARN: handler generated error: {}", e),
			},
			None => {}
		}

		if payload_len + offset == len {
			handle.inner.rbuf.clear();
		} else {
			// SAFETY: we know that n < len so there will be no error here
			let _ = handle.inner.rbuf.shift(payload_len + offset);
		}
	}

	fn close_cleanly(handle: &mut Box<Connection>, status: u16) {
		let conn = Connection {
			inner: handle.inner.clone().unwrap(),
		};
		let resp = WsResponse { conn };
		resp.close(status);
	}

	fn proc_messages(ctx: &mut WsContext, conn: &mut Box<Connection>) {
		loop {
			let slen = conn.inner.rbuf.len();
			match conn.inner.cstate {
				ConnectionState::NeedHandshake => {
					if conn.inner.ctype == ConnectionType::ClientConnection {
						Self::proc_hs_client(conn)
					} else {
						Self::proc_hs(conn)
					}
				}
				_ => Self::proc_hs_complete(conn, ctx),
			}
			let elen = conn.inner.rbuf.len();
			if elen == 0 || elen == slen {
				break;
			}
		}
	}

	fn proc_write(ctx: &mut WsContext, conn: &mut Box<Connection>, ehandle: *const u8) {
		loop {
			let ret = safe_socket_send(
				&conn.inner.handle as *const u8,
				conn.inner.wbuf[0..conn.inner.wbuf.len()].as_ptr(),
				conn.inner.wbuf.len(),
			);
			if ret < 0 {
				if ret != EAGAIN.into() {
					safe_socket_shutdown(&conn.inner.handle as *const u8);
				}
				break;
			} else {
				if ret > 0 {
					// cannot be an error
					let _ = conn.inner.wbuf.shift(ret as usize);
					let nlen = conn.inner.wbuf.len();
					// downward resize cannot be an error
					let _ = conn.inner.wbuf.resize(nlen);
				} else {
					break;
				}
			}
		}

		if conn.inner.wbuf.len() == 0 {
			// cancel loop
			safe_socket_multiplex_unregister_write(
				&ctx.state.wstate[ctx.tid].mplex as *const u8,
				ehandle,
			);
		}
	}

	fn proc_read(ctx: &mut WsContext, conn: &mut Box<Connection>, ehandle: *const u8) {
		loop {
			let rlen = conn.inner.rbuf.len();
			match conn.inner.rbuf.resize(rlen + 256) {
				Ok(_) => {}
				Err(_e) => {
					println!("WARN: Could not allocate read buffer! Closing connection.");
					safe_socket_shutdown(ehandle);
					break;
				}
			}
			let buf = &mut conn.inner.rbuf[rlen..rlen + 256];
			let len = safe_socket_recv(ehandle, buf.as_mut_ptr(), 256);

			if len == 0 || (len < 0 && len != EAGAIN as i64) {
				{
					let mut conn_inner = conn.inner.clone().unwrap();
					let _l = conn.inner.lock.write();
					conn_inner.cstate = ConnectionState::Closed;
				}
				safe_socket_close(ehandle);
				Self::remove_from_list(ctx, conn);
				conn.unleak();

				break;
			} else if len < 0 {
				if rlen == 0 {
					conn.inner.rbuf.clear();
				} else {
					conn.inner.rbuf.resize(rlen).unwrap();
				}
				// EAGAIN
				break;
			}

			conn.inner.rbuf.resize(len as usize + rlen).unwrap();
			if len <= 0 {
				break;
			} else {
				Self::proc_messages(ctx, conn);
			}
		}
	}

	fn proc_accept(ctx: &mut WsContext, conn: &mut Box<Connection>, ehandle: *const u8) {
		let mplex = ctx.state.wstate[ctx.tid].mplex;
		loop {
			let mut handle = [0u8; 4];
			let nhandle: *mut u8 = &mut handle as *mut u8;
			let res = safe_socket_accept(ehandle, nhandle);
			if res < 0 {
				if res == EAGAIN {
					break;
				} else {
					println!("WARN: Error accepting socket code: {}", res);
					break;
				}
			}
			let connection = match Connection::new(
				ConnectionType::ServerConnection,
				handle,
				ctx.state.wstate[ctx.tid].send.clone().unwrap(),
				ctx.state.config.debug_pending,
				ctx.state.wstate[ctx.tid].wakeup,
			) {
				Ok(connection) => connection,
				Err(_e) => {
					continue;
				}
			};
			let mut boxed_conn = match Box::new(connection) {
				Ok(b) => b,
				Err(_e) => {
					continue;
				}
			};
			boxed_conn.inner.connptr = boxed_conn.as_ptr();
			boxed_conn.leak();

			if safe_socket_multiplex_register(
				&mplex as *const u8,
				nhandle,
				REG_READ_FLAG,
				boxed_conn.as_ptr().raw() as *const u8,
			) < 0
			{
				println!("WARN: could not register accepted connection!");
				safe_socket_close(nhandle);
			}

			Self::update_head(ctx, &mut boxed_conn);
		}
	}

	fn proc_connection(
		ctx: &mut WsContext,
		conn: &mut Box<Connection>,
		ehandle: *const u8,
		evt: *const u8,
	) {
		match &conn.inner.ctype {
			ConnectionType::Server => {
				// since we are edge triggered, no other events
				// can fire until we accept the connections, so
				// we know this can only happen in each thread once
				let cur = aload!(&ctx.state.itt);
				let rem = rem_usize(cur as usize, ctx.state.config.threads as usize);
				if ctx.state.config.threads != 0 && rem == ctx.tid as usize {
					Self::proc_accept(ctx, conn, ehandle);
					aadd!(&mut ctx.state.itt, 1);
				}
			}
			_ => {
				if safe_socket_event_is_read(evt) {
					Self::proc_read(ctx, conn, ehandle);
				} else {
					let conn2 = conn.clone().unwrap();
					let _l = conn2.inner.lock.write();
					Self::proc_write(ctx, conn, ehandle);
				}
			}
		}
	}

	fn event_loop(ctx: &mut WsContext) -> Result<(), Error> {
		let mut ehandle = [0u8; 4];
		let ehandle: *mut u8 = &mut ehandle as *mut u8;
		let wakeup = &ctx.state.wstate[ctx.tid].wakeup as *const u8;
		let mplex = &ctx.state.wstate[ctx.tid].mplex as *const u8;

		loop {
			let count = safe_socket_multiplex_wait(mplex, ctx.events, ctx.state.config.max_events);
			{
				let _l = ctx.state.lock.read();
				if ctx.state.halt {
					break;
				}
			}
			for i in 0..count {
				let evt = unsafe {
					ctx.events
						.add(i as usize * safe_socket_event_size() as usize)
				};
				safe_socket_event_handle(ehandle, evt);

				if safe_socket_handle_eq(ehandle, wakeup) {
					safe_socket_clear_pipe(ehandle);
					Self::proc_wakeup(ctx);
				} else {
					let ptr = safe_socket_event_ptr(evt) as *const ConnectionInner;
					let mut connection = Box::from_raw(Ptr::new(ptr as *mut Connection));
					connection.leak();
					let ehandle = &connection.inner.handle as *const u8;
					Self::proc_connection(ctx, &mut connection, ehandle, evt);
				}
			}
		}

		// cleanup connections
		let mut cur = ctx.state.wstate[ctx.tid].head;
		while !cur.is_null() {
			let v = cur;
			cur = unsafe { (*cur).inner.next.raw() };
			let b = Box::from_raw(Ptr::new(v));
			if b.inner.ctype != ConnectionType::Server || ctx.tid == 0 {
				safe_socket_close(&b.inner.handle as *const u8);
			}
		}

		safe_socket_close(&ctx.state.wstate[ctx.tid].wakeup as *const u8);
		unsafe {
			safe_socket_close((&ctx.state.wstate[ctx.tid].wakeup as *const u8).add(4));
		}
		safe_socket_close(&ctx.state.wstate[ctx.tid].mplex as *const u8);

		safe_release(ctx.events);

		Ok(())
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use core::str::from_utf8_unchecked;

	#[test]
	fn test_ws1() {
		let initial = crate::sys::safe_getalloccount();
		let initial_fds = crate::sys::safe_getfdcount();
		{
			let config = WsConfig {
				threads: 4,
				..WsConfig::default()
			};
			let mut ws = WebSocket::new(config).unwrap();
			let lock = lock_box!().unwrap();
			let mut conf = Rc::new(false).unwrap();
			let lock_clone = lock.clone().unwrap();
			let conf_clone = conf.clone().unwrap();
			ws.start().unwrap();

			let b: Box<dyn FnMut(WsRequest, WsResponse) -> Result<(), Error>> =
				Box::new(move |req: WsRequest, mut resp: WsResponse| {
					let s = unsafe { from_utf8_unchecked(&req.msg()[0..req.msg().len()]) };
					println!("s={}", s);
					if s == "this is a test" {
						let _ = resp.send("got it!");
					} else if s == "got it!" {
						let _l = lock.write();
						*conf = true;
					}
					Ok(())
				})
				.unwrap();
			ws.register_handler(b);

			let port = ws
				.add_server(WsServerConfig {
					addr: [127, 0, 0, 1],
					port: 9999,
					backlog: 10,
				})
				.unwrap();
			ws.stop().unwrap();
		}
		assert_eq!(initial, crate::sys::safe_getalloccount());
		assert_eq!(initial_fds, crate::sys::safe_getfdcount());
	}

	#[test]
	fn test_ws2() {
		let initial = crate::sys::safe_getalloccount();
		let initial_fds = crate::sys::safe_getfdcount();
		{
			let config = WsConfig {
				threads: 4,
				..WsConfig::default()
			};
			let mut ws = WebSocket::new(config).unwrap();
			let lock = lock_box!().unwrap();
			let mut conf = Rc::new(false).unwrap();
			let lock_clone = lock.clone().unwrap();
			let conf_clone = conf.clone().unwrap();
			ws.start().unwrap();

			let b: Box<dyn FnMut(WsRequest, WsResponse) -> Result<(), Error>> =
				Box::new(move |req: WsRequest, mut resp: WsResponse| {
					let s = unsafe { from_utf8_unchecked(&req.msg()[0..req.msg().len()]) };
					if s == "this is a test" {
						let _ = resp.send("got it!");
					} else if s == "got it!" {
						let _l = lock.write();
						*conf = true;
					}
					Ok(())
				})
				.unwrap();
			let _ = ws.register_handler(b);

			let port = ws
				.add_server(WsServerConfig {
					addr: [127, 0, 0, 1],
					port: 0,
					backlog: 10,
				})
				.unwrap();

			let mut req = ws
				.add_client(WsClientConfig {
					addr: [127, 0, 0, 1],
					port,
				})
				.unwrap();

			assert!(req.send("this is a test").is_ok());

			loop {
				{
					let _l = lock_clone.read();
					if *conf_clone {
						break;
					}
				}
				crate::sys::safe_sleep_millis(1);
			}

			ws.stop().unwrap();
		}
		assert_eq!(initial, crate::sys::safe_getalloccount());
		assert_eq!(initial_fds, crate::sys::safe_getfdcount());
	}

	#[test]
	fn test_ws_perf() {
		let initial = crate::sys::safe_getalloccount();
		let initial_fds = crate::sys::safe_getfdcount();
		{
			let config = WsConfig {
				threads: 8,
				..WsConfig::default()
			};
			let threads = 4;
			let target = 1_000;

			let mut ws = WebSocket::new(config).unwrap();
			ws.start().unwrap();
			let mut count = Rc::new([0u64; 256]).unwrap();
			let count_clone = count.clone().unwrap();
			let mut sends = Vec::new();
			let mut recvs = Vec::new();
			for _i in 0..threads {
				let (send, recv) = channel().unwrap();
				let _ = sends.push(send);
				let _ = recvs.push(recv);
			}

			let b: Box<dyn FnMut(WsRequest, WsResponse) -> Result<(), Error>> =
				Box::new(move |req: WsRequest, _resp: WsResponse| {
					let msg = req.msg();
					let item = from_be_bytes_u64(&msg[1..9]);

					let index = msg[0];
					assert_eq!((*count)[index as usize], item);
					(*count)[index as usize] += 1;
					if (*count)[index as usize] == target {
						let _ = sends[index as usize].send(());
					}

					Ok(())
				})
				.unwrap();
			let _ = ws.register_handler(b);

			let port = ws
				.add_server(WsServerConfig {
					addr: [127, 0, 0, 1],
					port: 0,
					backlog: 10,
				})
				.unwrap();
			let mut resps = Vec::new();
			for _i in 0..threads {
				let resp = ws
					.add_client(WsClientConfig {
						addr: [127, 0, 0, 1],
						port,
					})
					.unwrap();
				let _ = resps.push(resp);
			}

			let config = RuntimeConfig {
				min_threads: threads * 2,
				max_threads: threads * 2,
			};
			let mut runtime = Runtime::<()>::new(config).unwrap();
			assert!(runtime.start().is_ok());

			let mut jhs = Vec::new();

			for v in 0..threads {
				let mut resp = resps[v as usize].clone().unwrap();
				let h = runtime
					.execute(move || {
						let mut bytes = [b'm'; 10];
						bytes[0] = v as u8;
						for i in 0..target {
							to_be_bytes_u64(i as u64, &mut bytes[1..9]);
							assert!(resp.sendb(&bytes).is_ok());
						}
					})
					.unwrap();
				let _ = jhs.push(h);
			}

			for i in 0..jhs.len() {
				jhs[i].block_on();
			}

			for i in 0..threads {
				let _ = recvs[i as usize].recv();
				assert_eq!((*count_clone)[i as usize], target);
			}

			ws.stop().unwrap();
		}
		assert_eq!(initial, crate::sys::safe_getalloccount());
		assert_eq!(initial_fds, crate::sys::safe_getfdcount());
	}

	#[test]
	fn test_ws_pending() {
		let initial = crate::sys::safe_getalloccount();
		let initial_fds = crate::sys::safe_getfdcount();
		{
			let config = WsConfig {
				threads: 4,
				debug_pending: true,
				..WsConfig::default()
			};
			let mut ws = WebSocket::new(config).unwrap();
			let lock = lock_box!().unwrap();
			let mut conf = Rc::new(false).unwrap();
			let lock_clone = lock.clone().unwrap();
			let conf_clone = conf.clone().unwrap();
			ws.start().unwrap();

			let b: Box<dyn FnMut(WsRequest, WsResponse) -> Result<(), Error>> =
				Box::new(move |req: WsRequest, mut resp: WsResponse| {
					let s = unsafe { from_utf8_unchecked(&req.msg()[0..req.msg().len()]) };
					if s == "this is a test" {
						let _ = resp.send("got it!");
					} else if s == "got it!" {
						let _l = lock.write();
						*conf = true;
					}
					Ok(())
				})
				.unwrap();
			let _ = ws.register_handler(b);
			let port = ws
				.add_server(WsServerConfig {
					addr: [127, 0, 0, 1],
					port: 0,
					backlog: 10,
				})
				.unwrap();

			let mut req = ws
				.add_client(WsClientConfig {
					addr: [127, 0, 0, 1],
					port,
				})
				.unwrap();

			assert!(req.send("this is a test").is_ok());

			loop {
				{
					let _l = lock_clone.read();
					if *conf_clone {
						break;
					}
				}
				crate::sys::safe_sleep_millis(1);
			}

			ws.stop().unwrap();
		}
		assert_eq!(initial, crate::sys::safe_getalloccount());
		assert_eq!(initial_fds, crate::sys::safe_getfdcount());
	}
}
