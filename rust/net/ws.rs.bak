use core::cell::UnsafeCell;
use core::default::Default;
use core::ptr::{copy_nonoverlapping, drop_in_place};
use core::slice::from_raw_parts;
use prelude::*;
use sys::*;

const EAGAIN: i32 = -11;
const REG_READ_FLAG: i32 = 0x1;
const REG_WRITE_FLAG: i32 = 0x2;

const BAD_REQUEST: &str = "HTTP/1.1 400 Bad Request\r\n\
Content-Type: text/plain\r\n\
Connection: close\r\n\r\n";
const SWITCH_PROTOCOL: &str = "HTTP/1.1 101 Switching Protocols\r\n\
Upgrade: websocket\r\n\
Connection: Upgrade\r\n\
Sec-WebSocket-Accept: ";

const GET_PREFIX: &[u8] = "GET /".as_bytes();
const SEC_KEY_PREFIX: &[u8] = "Sec-WebSocket-Key: ".as_bytes();

struct Handler {
	path: String,
	handler: Option<Box<dyn FnMut(WsMessage, WsResponse) -> Result<(), Error>>>,
}

impl PartialEq for Handler {
	fn eq(&self, other: &Handler) -> bool {
		strcmp(other.path.to_str(), self.path.to_str()) == 0
	}
}

impl Hash for Handler {
	fn hash(&self) -> usize {
		murmur3_32_of_slice(self.path.to_str().as_bytes(), get_murmur_seed()) as usize
	}
}

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

pub struct WsMessage<'a> {
	msg: &'a [u8],
	path: String,
}

impl WsMessage<'_> {
	pub fn msg(&self) -> &[u8] {
		let len = self.msg.len();
		unsafe { from_raw_parts(self.msg.as_ptr(), len) }
	}

	pub fn path(&self) -> String {
		// SAFETY: String::clone cannot fail as it clones an Rc.
		self.path.clone().unwrap()
	}
}

pub struct WsResponse {
	wshandle: WsHandle,
}

enum MessageType {
	Text,
	Binary,
}

impl WsResponse {
	pub fn send(&mut self, msg: &str) {
		self.send_impl(MessageType::Text, msg.as_bytes());
	}

	pub fn sendb(&mut self, msg: &[u8]) {
		self.send_impl(MessageType::Binary, msg);
	}

	pub fn close(&mut self, status_code: u16) {
		let status_code = to_be_bytes_u16(status_code);
		self.wshandle.inner.wh.writeb(&[0x88, 0]);

		self.wshandle.inner.wh.writeb(&[0x88, 2]);
		self.wshandle.inner.wh.writeb(&status_code);
	}

	fn send_impl(&mut self, mtype: MessageType, bytes: &[u8]) {
		let b1 = match mtype {
			MessageType::Text => 0x81,
			MessageType::Binary => 0x82,
		};

		if bytes.len() <= 125 {
			self.wshandle.inner.wh.writeb(&[b1, bytes.len() as u8]);
		} else if bytes.len() <= 65535 {
			self.wshandle.inner.wh.writeb(&[b1, 126]);
			let len = to_be_bytes_u16(bytes.len() as u16);
			self.wshandle.inner.wh.writeb(&len);
		} else {
			self.wshandle.inner.wh.writeb(&[b1, 127]);
			let len = to_be_bytes_u64(bytes.len() as u64);
			self.wshandle.inner.wh.writeb(&len);
		}

		self.wshandle.inner.wh.writeb(bytes);
	}
}

enum ConnectionState {
	NeedHandshake,
	HandshakeComplete(String),
	//Closed,
}

pub struct WriteHandle {
	buffer: Vec<u8>,
	lock: Lock,
	handle: [u8; 4],
	mplex: [u8; 4],
	state: ConnectionState,
}

impl WriteHandle {
	pub fn writeb(&mut self, msg: &[u8]) {
		let _l = self.lock.write();
		let mut res = if self.buffer.len() == 0 {
			safe_socket_send(&self.handle as *const u8, msg.as_ptr(), msg.len())
		} else {
			0
		};
		if res == EAGAIN.into() || (res >= 0 && (res as usize) < msg.len()) {
			if res < 0 {
				res = 0;
			}
			unsafe {
				match self
					.buffer
					.append_ptr(msg.as_ptr().add(res as usize), msg.len() - (res as usize))
				{
					Ok(_) => {}
					Err(_e) => {
						// could not allocate space to append data to buffer. Close socket.
						println!(
							"WARN: Could not allocate space to write buffer. Dropping connection!"
						);
						safe_socket_shutdown(&self.handle as *const u8);
					}
				}
			}
			if safe_socket_multiplex_register(
				&mut self.mplex as *mut u8,
				&mut self.handle as *mut u8,
				REG_WRITE_FLAG,
			) < 0
			{
				safe_socket_shutdown(&self.handle as *const u8);
			}
		} else if res < 0 {
			safe_socket_shutdown(&self.handle as *const u8);
		}
	}
	pub fn write(&mut self, msg: &str) {
		self.writeb(msg.as_bytes())
	}
}

pub struct HandleInner {
	read: Vec<u8>,
	wh: WriteHandle,
}

pub struct WsHandle {
	inner: Rc<HandleInner>,
}

impl WsHandle {
	fn new(handle: [u8; 4], mplex: [u8; 4]) -> Result<Self, Error> {
		let mut buffer = Vec::new();
		buffer.set_min(512);
		let wh = WriteHandle {
			buffer,
			lock: lock!(),
			handle,
			mplex,
			state: ConnectionState::NeedHandshake,
		};
		let mut read = Vec::new();
		read.set_min(512);
		let inner = HandleInner { read, wh };
		let inner = match Rc::new(inner) {
			Ok(inner) => inner,
			Err(e) => return Err(e),
		};
		Ok(WsHandle { inner })
	}
}

impl PartialEq for WsHandle {
	fn eq(&self, other: &WsHandle) -> bool {
		self.inner.wh.handle == other.inner.wh.handle
	}
}

impl Hash for WsHandle {
	fn hash(&self) -> usize {
		murmur3_32_of_slice(&self.inner.wh.handle, get_murmur_seed()) as usize
	}
}

pub struct GlobalState {
	server: [u8; 4],
	jhs: UnsafeCell<Vec<JoinHandle>>,
	mplexes: Vec<[u8; 4]>,
	handlers: Hashtable<Handler>,
	wakeup: [u8; 4],
	stop: u64,
	config: WsConfig,
	port: u16,
	itt: u64,
	lock: Lock,
}

pub struct WsServer {
	global_state: Rc<GlobalState>,
}

pub struct WsContext {
	state: Rc<GlobalState>,
	handles: Hashtable<WsHandle>,
	tid: u64,
	mplex: [u8; 4],
	events: *mut u8,
	fhandle: WsHandle,
}

impl WsContext {
	fn new(mut state: Rc<GlobalState>, tid: u64) -> Result<Self, Error> {
		let handles = match Hashtable::new(1024) {
			Ok(handles) => handles,
			Err(e) => return Err(e),
		};
		let events =
			safe_alloc(safe_socket_event_size() * state.config.max_events as usize) as *mut u8;
		let mut mplex = [0u8; 4];
		if safe_socket_multiplex_init(&mut mplex as *mut u8) < 0 {
			return Err(err!(CreateFileDescriptor));
		}

		if safe_socket_multiplex_register(
			&mut mplex as *mut u8,
			&mut state.server as *mut u8,
			REG_READ_FLAG,
		) < 0
		{
			return Err(err!(MultiplexRegister));
		}

		let fhandle = match WsHandle::new([0u8; 4], [0u8; 4]) {
			Ok(fhandle) => fhandle,
			Err(e) => return Err(e),
		};

		Ok(Self {
			state,
			handles,
			tid,
			events,
			mplex,
			fhandle,
		})
	}
}

impl WsServer {
	pub fn new(config: WsConfig) -> Result<Self, Error> {
		let itt = 0;
		let handlers = match Hashtable::new(1024) {
			Ok(handlers) => handlers,
			Err(e) => return Err(e),
		};

		let jhs: UnsafeCell<Vec<JoinHandle>> = Vec::new().into();
		let port = config.port;
		let lock = lock!();
		let stop = 0;
		let server = [0u8; 4];
		let wakeup = [0u8; 4];
		let mplexes = Vec::new();

		let global_state = match Rc::new(GlobalState {
			config,
			itt,
			mplexes,
			handlers,
			jhs,
			port,
			lock,
			stop,
			server,
			wakeup,
		}) {
			Ok(global_state) => global_state,
			Err(e) => return Err(e),
		};

		Ok(WsServer { global_state })
	}

	pub fn add_client(&mut self, host: [u8; 4], port: u16) -> Result<WsResponse, Error> {
		let mut handle = [0u8; 4];
		if safe_socket_connect(&mut handle as *mut u8, &host as *const u8, port.into()) < 0 {
			return Err(err!(SocketConnect));
		}

		let next = rem_usize(
			aadd!(&mut self.global_state.itt, 1) as usize,
			self.global_state.config.threads,
		);
		let mut mplex = self.global_state.mplexes[next];
		safe_socket_multiplex_register(
			&mut mplex as *mut u8,
			&mut handle as *mut u8,
			REG_READ_FLAG,
		);

		let wshandle = match WsHandle::new(handle, mplex) {
			Ok(wshandle) => wshandle,
			Err(e) => {
				safe_socket_close(&handle as *const u8);
				return Err(e);
			}
		};

		Ok(WsResponse { wshandle })
	}

	pub fn port(&self) -> u16 {
		self.global_state.port
	}

	pub fn register_handler(
		&mut self,
		path: &str,
		handler: Box<dyn FnMut(WsMessage, WsResponse) -> Result<(), Error>>,
	) -> Result<(), Error> {
		let handler = Handler {
			path: String::new(path).unwrap(),
			handler: Some(handler),
		};
		let ptr = Ptr::alloc(Node::new(handler)).unwrap();
		self.global_state.handlers.insert(ptr);
		Ok(())
	}

	pub fn start(&mut self) -> Result<(), Error> {
		match self.bind_socket() {
			Ok(_) => {}
			Err(e) => return Err(e),
		}

		let global_state = match self.global_state.clone() {
			Ok(global_state) => global_state,
			Err(e) => return Err(e),
		};

		Self::start_threads(global_state)
	}

	fn proc_accept(ehandle: *const u8, ctx: &mut WsContext) {
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
			if safe_socket_multiplex_register(&mut ctx.mplex as *mut u8, nhandle, REG_READ_FLAG) < 0
			{
				println!("WARN: could not register accepted connection!");
				safe_socket_close(nhandle);
			}

			let mut handle = [0u8; 4];
			unsafe {
				copy_nonoverlapping(nhandle, &mut handle as *mut u8, 4);
			}

			let conn =
				match WsHandle::new(handle, ctx.mplex) {
					Ok(conn) => conn,
					Err(_e) => {
						println!("WARN: Could not allocate memory for new connection! Closing connection.");
						safe_socket_close(nhandle);
						continue;
					}
				};

			let conn =
				match Ptr::alloc(Node::new(conn)) {
					Ok(conn) => conn,
					Err(_e) => {
						println!("WARN: Could not allocate memory for new connection! Closing connection.");
						safe_socket_close(nhandle);
						continue;
					}
				};
			ctx.handles.insert(conn);
		}
	}

	fn bad_request(handle: &mut WsHandle) {
		handle.inner.wh.write(BAD_REQUEST);
		safe_socket_shutdown(&mut handle.inner.wh.handle as *const u8);
	}

	fn handle_websocket_handshake(sec_key: &[u8]) -> [u8; 28] {
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

	fn switch_protocol(handle: &mut WsHandle, accept_key: &[u8; 28]) {
		handle.inner.wh.write(SWITCH_PROTOCOL);
		handle.inner.wh.writeb(accept_key);
		handle.inner.wh.write("\r\n\r\n");
	}

	fn proc_hs(handle: &mut WsHandle) {
		let len = handle.inner.read.len();
		let rvec = &handle.inner.read;
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
			use core::str::from_utf8_unchecked;
			let uri = unsafe { String::new(from_utf8_unchecked(uri)).unwrap() };

			let mut sec_key: &[u8] = &[];

			for i in uri_end..len {
				if rvec[i] == b'\n'
					&& rvec[i - 1] == b'\r'
					&& rvec[i - 2] == b'\n'
					&& rvec[i - 3] == b'\r'
				{
					if sec_key == &[] {
						Self::bad_request(handle);
					} else {
						let accept_key = Self::handle_websocket_handshake(sec_key);
						Self::switch_protocol(handle, &accept_key);
						handle.inner.read.clear();
						handle.inner.wh.state = ConnectionState::HandshakeComplete(uri);
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

	fn close_cleanly(handle: &mut WsHandle, status: u16) {
		let wshandle = WsHandle {
			inner: handle.inner.clone().unwrap(),
		};
		let mut resp = WsResponse { wshandle };
		resp.close(status);
		safe_socket_shutdown(&handle.inner.wh.handle as *const u8);
	}

	fn proc_hs_complete(handle: &mut WsHandle, state: Rc<GlobalState>) {
		let wshandle = WsHandle {
			inner: handle.inner.clone().unwrap(),
		};

		let path = match &handle.inner.wh.state {
			ConnectionState::HandshakeComplete(s) => s.clone().unwrap(),
			_ => String::new("unknown").unwrap(),
		};

		let len = handle.inner.read.len();

		// min length to try to process
		if len < 2 {
			return;
		}

		let rvec = &mut handle.inner.read;
		let _fin = rvec[0] & 0x80 != 0;

		// reserved bits not 0
		if rvec[0] & 0x70 != 0 {
			Self::close_cleanly(handle, 1002);
			return;
		}

		let _op = rvec[0] & !0x80;
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

		println!("path={}", path);
		let wsmsg = WsMessage {
			msg: payload,
			path: path.clone().unwrap(),
		};

		let _res = match state.handlers.find(&Handler {
			path,
			handler: None,
		}) {
			Some(mut handler) => match &mut handler.handler {
				Some(ref mut callback) => callback(wsmsg, WsResponse { wshandle }),
				None => Ok(()),
			},
			None => Ok(()),
		};

		if payload_len + offset == len {
			handle.inner.read.clear();
		} else {
			// TODO: handle err
			let _ = handle.inner.read.shift(payload_len + offset);
		}
	}

	fn proc_messages(handle: &mut WsHandle, state: Rc<GlobalState>) {
		match handle.inner.wh.state {
			ConnectionState::NeedHandshake => Self::proc_hs(handle),
			_ => Self::proc_hs_complete(handle, state),
		}
	}

	fn proc_read(ehandle: *const u8, ctx: &mut WsContext) -> usize {
		unsafe {
			copy_nonoverlapping(
				ehandle,
				ctx.fhandle.inner.wh.handle.as_mut_ptr(),
				ctx.fhandle.inner.wh.handle.len(),
			);
		}
		let mut handle = match ctx.handles.find(&ctx.fhandle) {
			Some(handle) => handle,
			None => {
				exit!("Could not find connection handle!");
			}
		};

		let rlen = handle.inner.read.len();
		match handle.inner.read.resize(rlen + 256) {
			Ok(_) => {}
			Err(_e) => {
				println!("WARN: Could not allocate read buffer! Closing connection.");
				safe_socket_shutdown(ehandle);
				return 0;
			}
		}
		let buf = &mut handle.inner.read[rlen..rlen + 256];
		let len = safe_socket_recv(ehandle, buf.as_mut_ptr(), 256);

		if len == 0 || (len < 0 && len != EAGAIN as i64) {
			safe_socket_close(ehandle);
			let to_drop = match ctx.handles.remove(&ctx.fhandle) {
				Some(to_drop) => to_drop,
				None => exit!("could not remove handle from hashtable!"),
			};
			unsafe {
				drop_in_place(to_drop.raw());
			}
			to_drop.release();
			return 0;
		} else if len < 0 {
			if rlen == 0 {
				handle.inner.read.clear();
			} else {
				// SAFETY: this is a downward resize which cannot fail.
				handle.inner.read.resize(rlen).unwrap();
			}
			// EAGAIN
			return 0;
		}

		// SAFETY: this is a downward resize which cannot fail.
		handle.inner.read.resize(len as usize + rlen).unwrap();
		if len <= 0 {
			0
		} else {
			// SAFETY: unwrap is ok because it's an Rc which cannot fail
			Self::proc_messages(&mut handle, ctx.state.clone().unwrap());
			len as usize
		}
	}

	fn proc_write(ehandle: *const u8, ctx: &mut WsContext) -> usize {
		unsafe {
			copy_nonoverlapping(
				ehandle,
				ctx.fhandle.inner.wh.handle.as_mut_ptr(),
				ctx.fhandle.inner.wh.handle.len(),
			);
		}
		let handle = match ctx.handles.find(&ctx.fhandle) {
			Some(handle) => handle,
			None => {
				exit!("Could not find connection handle!");
			}
		};

		let wlen = handle.inner.wh.buffer.len();
		let buf = &handle.inner.wh.buffer[0..wlen];
		let len = safe_socket_send(ehandle, buf.as_ptr(), wlen);

		if len < 0 {
			safe_socket_shutdown(ehandle);
			0
		} else {
			0
		}
	}

	fn thread_loop(mut ctx: WsContext) -> Result<(), Error> {
		let mut ehandle = [0u8; 4];
		let wakeup: *mut u8 = &mut ctx.state.wakeup as *mut u8;
		let server: *mut u8 = &mut ctx.state.server as *mut u8;

		let ehandle: *mut u8 = &mut ehandle as *mut u8;
		let mut stop = false;
		loop {
			let count = safe_socket_multiplex_wait(
				&mut ctx.mplex as *mut u8,
				ctx.events,
				ctx.state.config.max_events,
			);
			for i in 0..count {
				let evt = unsafe {
					ctx.events
						.add(i as usize * safe_socket_event_size() as usize)
				};
				safe_socket_event_handle(ehandle, evt);

				if safe_socket_handle_eq(ehandle, server) {
					// since we are edge triggered, no other events
					// can fire until we accept the connections, so
					// we know this can only happen in each thread once
					let cur = aload!(&ctx.state.itt);
					let rem = rem_usize(cur as usize, ctx.state.config.threads);
					if ctx.state.config.threads != 0 && rem == ctx.tid as usize {
						Self::proc_accept(ehandle, &mut ctx);
						aadd!(&mut ctx.state.itt, 1);
					}
				} else if safe_socket_handle_eq(ehandle, wakeup) {
					if aload!(&ctx.state.stop) != 0 {
						stop = true;
						break;
					}
				} else if safe_socket_event_is_read(evt) {
					while Self::proc_read(ehandle, &mut ctx) != 0 {}
				} else if safe_socket_event_is_write(evt) {
					while Self::proc_write(ehandle, &mut ctx) != 0 {}
				}
			}
			if stop {
				for element in ctx.handles {
					unsafe {
						drop_in_place(element.raw());
					}
					element.release();
				}
				safe_release(ctx.events);
				break;
			}
		}

		Ok(())
	}

	fn start_threads(global_state: Rc<GlobalState>) -> Result<(), Error> {
		for tid in 0..global_state.config.threads {
			let mut global_state = match global_state.clone() {
				Ok(global_state) => global_state,
				Err(e) => return Err(e),
			};
			let global_state_clone = match global_state.clone() {
				Ok(global_state) => global_state,
				Err(e) => return Err(e),
			};

			let ctx = match WsContext::new(global_state_clone, tid as u64) {
				Ok(ctx) => ctx,
				Err(e) => return Err(e),
			};

			match global_state.mplexes.push(ctx.mplex) {
				Ok(_) => {}
				Err(e) => return Err(e),
			}

			let s = spawnj(move || match Self::thread_loop(ctx) {
				Ok(_) => {}
				Err(e) => println!("err={}", e),
			});

			let _l = global_state.lock.write();
			match s {
				Ok(jh) => match unsafe { (*global_state.jhs.get()).push(jh) } {
					Ok(_) => {}
					Err(e) => return Err(e),
				},
				Err(e) => return Err(e),
			}
		}
		Ok(())
	}

	fn bind_socket(&mut self) -> Result<(), Error> {
		let server_ptr = &mut self.global_state.server as *mut u8;
		self.global_state.port = safe_socket_listen(
			server_ptr,
			self.global_state.config.addr.as_ptr(),
			self.port(),
			self.global_state.config.backlog,
		) as u16;

		Ok(())
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use core::str::from_utf8_unchecked;

	#[test]
	fn test_ws1() {
		let config = WsConfig {
			port: 9999,
			..Default::default()
		};

		let lock = lock_box!().unwrap();
		let lock2 = lock.clone().unwrap();
		let mut ws = WsServer::new(config).unwrap();
		let b: Box<dyn FnMut(WsMessage, WsResponse) -> Result<(), Error>> =
			Box::new(move |msg: WsMessage, mut resp: WsResponse| {
				let _v = lock.write();
				let x = unsafe { from_utf8_unchecked(&msg.msg[0..msg.msg.len()]) };
				println!("in handler[{}]. Msg={}", msg.path, x);
				resp.send("got it!");
				Ok(())
			})
			.unwrap();
		let _ = ws.register_handler("/abc", b);

		let b: Box<dyn FnMut(WsMessage, WsResponse) -> Result<(), Error>> =
			Box::new(move |msg: WsMessage, mut resp: WsResponse| {
				let _v = lock2.write();
				let x = unsafe { from_utf8_unchecked(&msg.msg[0..msg.msg.len()]) };
				println!("in handler2[{}]. Msg={}", msg.path, x);
				resp.send("got it!");
				Ok(())
			})
			.unwrap();
		let _ = ws.register_handler("/def", b);

		ws.start().unwrap();
		let port = ws.port();
		ws.add_client([0u8; 4], port).unwrap();

		let handle = safe_alloc(safe_socket_handle_size()) as *mut u8;
		let addr = [127u8, 0u8, 0u8, 1u8];
		safe_socket_connect(handle, &addr as *const u8, ws.port() as i32);

		//park();

		safe_release(handle);
	}
}
