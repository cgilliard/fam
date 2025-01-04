#![allow(dead_code)]
#![allow(unused_variables)]

use core::ptr::{copy_nonoverlapping, null_mut};
use prelude::*;
use sys::{
	safe_alloc, safe_pipe, safe_release, safe_socket_accept, safe_socket_close,
	safe_socket_event_handle, safe_socket_event_ptr, safe_socket_event_size, safe_socket_handle_eq,
	safe_socket_listen, safe_socket_multiplex_init, safe_socket_multiplex_register,
	safe_socket_multiplex_wait, safe_socket_send,
};

const EAGAIN: i32 = -11;
const REG_READ_FLAG: i32 = 0x1;
const REG_WRITE_FLAG: i32 = 0x2;

struct Handler {
	path: String,
	handler: Option<Box<dyn FnMut(WsRequest, WsResponse) -> Result<(), Error>>>,
}

pub struct WsConfig {
	threads: u64,
	max_events: i32,
	conn_hash_size: usize,
}

pub struct WsMessage<'a> {
	msg: &'a [u8],
	path: String,
}

enum ConnectionState {
	NeedHandshake,
	HandshakeComplete(String),
	Closed,
}

enum ConnectionType {
	Server,
	ServerConnection,
	ClientConnection,
}

struct ConnectionInner {
	next: Ptr<Connection>,
	prev: Ptr<Connection>,
	ctype: ConnectionType,
	rbuf: Vec<u8>,
	wbuf: Vec<u8>,
	handle: [u8; 4],
	mplex: [u8; 4],
	is_open: bool,
	lock: Lock,
}

struct Connection {
	inner: Rc<ConnectionInner>,
}

impl Clone for Connection {
	fn clone(&self) -> Result<Self, Error> {
		// SAFETY: rc clone does not fail
		Ok(Self {
			inner: self.inner.clone().unwrap(),
		})
	}
}

impl Connection {
	fn new(ctype: ConnectionType, handle: [u8; 4], mplex: [u8; 4]) -> Result<Self, Error> {
		match Rc::new(ConnectionInner {
			next: Ptr::null(),
			prev: Ptr::null(),
			ctype,
			rbuf: Vec::new(),
			wbuf: Vec::new(),
			handle,
			mplex,
			is_open: false,
			lock: lock!(),
		}) {
			Ok(inner) => Ok(Self { inner }),
			Err(e) => Err(e),
		}
	}
}

pub struct WsRequest<'a> {
	msg: WsMessage<'a>,
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

struct State {
	runtime: Option<Runtime<()>>,
	mplexes: Vec<[u8; 4]>,
	locks: Vec<LockBox>,
	heads: Vec<*mut Connection>,
	handlers: Hashtable<Handler>,
	wakeup: [u8; 8],
	config: WsConfig,
	itt: u64,
	lock: LockBox,
	halt: bool,
	threads_started: u64,
}

pub struct WsContext {
	state: Rc<State>,
	tid: u64,
	mplex: [u8; 4],
	events: *mut u8,
}

pub struct WsHandler {
	state: Rc<State>,
}

impl PartialEq for Handler {
	fn eq(&self, _: &Handler) -> bool {
		false
	}
}

impl Hash for Handler {
	fn hash(&self) -> usize {
		0
	}
}

impl PartialEq for Connection {
	fn eq(&self, _: &Connection) -> bool {
		false
	}
}

impl Hash for Connection {
	fn hash(&self) -> usize {
		0
	}
}

impl Default for WsConfig {
	fn default() -> Self {
		Self {
			threads: 4,
			max_events: 32,
			conn_hash_size: 1024,
		}
	}
}

impl State {
	fn new(config: WsConfig) -> Result<Self, Error> {
		let lock = match lock_box!() {
			Ok(lock) => lock,
			Err(e) => return Err(e),
		};
		match Hashtable::new(config.conn_hash_size) {
			Ok(handlers) => Ok(Self {
				runtime: None,
				mplexes: Vec::new(),
				handlers,
				wakeup: [0u8; 8],
				config,
				itt: 0,
				locks: Vec::new(),
				heads: Vec::new(),
				lock,
				halt: false,
				threads_started: 0,
			}),
			Err(e) => Err(e),
		}
	}
}

impl WsHandler {
	pub fn new(config: WsConfig) -> Result<Self, Error> {
		let state = match State::new(config) {
			Ok(state) => state,
			Err(e) => return Err(e),
		};
		// SAFETY: unwrap ok because Rc does not fail
		Ok(WsHandler {
			state: Rc::new(state).unwrap(),
		})
	}

	pub fn add_client(&mut self, config: WsClientConfig) -> Result<WsResponse, Error> {
		Err(err!(Todo))
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

		// SAFETY: clone on rc does not fail
		let mut state_clone = self.state.clone().unwrap();
		let mut state_clone2 = self.state.clone().unwrap();
		for mplex in &self.state.mplexes {
			let connection = match Connection::new(ConnectionType::Server, server, [0u8; 4]) {
				Ok(connection) => connection,
				Err(e) => return Err(e),
			};
			let mut boxed_conn = match Box::new(connection) {
				Ok(b) => b,
				Err(e) => return Err(e),
			};

			boxed_conn.leak();

			{
				let _l = self.state.locks[i].write();
				boxed_conn.inner.next = Ptr::new(self.state.heads[i]);
				boxed_conn.inner.prev = Ptr::null();
				if !self.state.heads[i].is_null() {
					unsafe {
						(*state_clone.heads[i]).inner.prev = Ptr::new(boxed_conn.as_ptr().raw());
					}
				}
				state_clone2.heads[i] = boxed_conn.as_ptr().raw();
			}

			if safe_socket_multiplex_register(
				mplex as *const u8,
				&server as *const u8,
				REG_READ_FLAG,
				boxed_conn.as_ptr().raw() as *mut u8,
			) < 0
			{
				safe_socket_close(server_ptr);
				return Err(err!(MultiplexRegister));
			}
			i += 1;
		}
		Ok(port as u16)
	}

	pub fn register_handler(
		&mut self,
		path: &str,
		handler: Box<dyn FnMut(WsMessage, WsResponse) -> Result<(), Error>>,
	) -> Result<(), Error> {
		Err(err!(Todo))
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

		if safe_pipe(&mut self.state.wakeup as *mut u8) < 0 {
			return Err(err!(Pipe));
		}

		let lock = match lock_box!() {
			Ok(lock) => lock,
			Err(e) => return Err(e),
		};

		for tid in 0..self.state.config.threads {
			// SAFETY: unwrap ok on rc.clone
			let mut state = self.state.clone().unwrap();
			let mut mplex = [0u8; 4];

			if safe_socket_multiplex_init(&mut mplex as *mut u8) < 0 {
				return Err(err!(CreateFileDescriptor));
			}

			match state.heads.push(null_mut()) {
				Ok(_) => {}
				Err(e) => return Err(e),
			}

			match state.mplexes.push(mplex) {
				Ok(_) => {}
				Err(e) => return Err(e),
			}

			let llock = match lock_box!() {
				Ok(l) => l,
				Err(e) => return Err(e),
			};
			match state.locks.push(llock) {
				Ok(_) => {}
				Err(e) => return Err(e),
			}

			if safe_socket_multiplex_register(
				&mplex as *const u8,
				&self.state.wakeup as *const u8,
				REG_READ_FLAG,
				null_mut(),
			) < 0
			{
				return Err(err!(MultiplexRegister));
			}

			let events =
				safe_alloc(safe_socket_event_size() * self.state.config.max_events as usize)
					as *mut u8;

			let mut ctx = WsContext {
				state,
				tid,
				mplex,
				events,
			};

			let _ = runtime.execute(move || match Self::event_loop(&mut ctx) {
				Ok(_) => {}
				Err(e) => println!("FATAL: unexpected error in event_loop: {}", e),
			});
		}

		self.state.runtime = Some(runtime);

		Ok(())
	}

	pub fn stop(&mut self) -> Result<(), Error> {
		// just in case all threads have not started spin here until that happens
		while aload!(&self.state.threads_started) != self.state.config.threads {}
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
			Some(ref mut rt) => {
				let ret = rt.stop();
				// close pipes
				if ret.is_ok() {
					safe_socket_close(&self.state.wakeup as *const u8);
					unsafe {
						safe_socket_close((&self.state.wakeup as *const u8).add(4));
					}
				}
				ret
			}
			None => Ok(()),
		}
	}

	fn wakeup_threads(&self) -> Result<(), Error> {
		if safe_socket_send(
			unsafe { (&self.state.wakeup as *const u8).add(4) },
			&b'0',
			1,
		) < 0
		{
			return Err(err!(WsStop));
		}
		Ok(())
	}

	fn proc_accept(ctx: &mut WsContext, conn: Box<Connection>, ehandle: *const u8) {
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

			let mut handle = [0u8; 4];
			unsafe {
				copy_nonoverlapping(nhandle, &mut handle as *mut u8, 4);
			}

			let connection =
				match Connection::new(ConnectionType::ServerConnection, handle, ctx.mplex) {
					Ok(connection) => connection,
					Err(e) => {
						continue;
					}
				};
			let mut boxed_conn = match Box::new(connection) {
				Ok(b) => b,
				Err(e) => {
					continue;
				}
			};

			boxed_conn.leak();

			if safe_socket_multiplex_register(
				&mut ctx.mplex as *mut u8,
				nhandle,
				REG_READ_FLAG,
				boxed_conn.as_ptr().raw() as *const u8,
			) < 0
			{
				println!("WARN: could not register accepted connection!");
				safe_socket_close(nhandle);
			}

			// SAFETY: rc clone doesn't fail
			let mut state_clone1 = ctx.state.clone().unwrap();
			let mut state_clone2 = ctx.state.clone().unwrap();
			{
				let _l = ctx.state.locks[ctx.tid as usize].write();
				boxed_conn.inner.next = Ptr::new(ctx.state.heads[ctx.tid as usize]);
				boxed_conn.inner.prev = Ptr::null();
				if !ctx.state.heads[ctx.tid as usize].is_null() {
					unsafe {
						(*state_clone1.heads[ctx.tid as usize]).inner.prev =
							Ptr::new(boxed_conn.as_ptr().raw());
					}
				}
				state_clone2.heads[ctx.tid as usize] = boxed_conn.as_ptr().raw();
			}
		}
	}

	fn proc_connection(ctx: &mut WsContext, conn: Box<Connection>, ehandle: *const u8) {
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
			_ => {}
		}
	}

	fn event_loop(ctx: &mut WsContext) -> Result<(), Error> {
		let mut ehandle = [0u8; 4];
		let ehandle: *mut u8 = &mut ehandle as *mut u8;
		let wakeup = &ctx.state.wakeup as *const u8;
		let mut stop = false;
		let mut first = true;

		while !stop {
			if first {
				first = false;
				aadd!(&mut ctx.state.threads_started, 1);
			}
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
				if safe_socket_handle_eq(ehandle, wakeup) {
					let _l = ctx.state.lock.read();
					if ctx.state.halt {
						stop = true;
						break;
					}
				} else {
					let ptr = safe_socket_event_ptr(ehandle);
					let mut connection = Box::from_raw(Ptr::new(ptr as *mut Connection));
					connection.leak();
					Self::proc_connection(ctx, connection, ehandle);
				}
			}
		}

		safe_socket_close(&mut ctx.mplex as *mut u8);
		safe_release(ctx.events);

		let _l = ctx.state.locks[ctx.tid as usize].write();
		let mut cur = ctx.state.heads[ctx.tid as usize];
		while !cur.is_null() {
			let v = cur;
			cur = unsafe { (*cur).inner.next.raw() };
			let b = Box::from_raw(Ptr::new(v));
		}

		Ok(())
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn test_ws1() {
		let initial = crate::sys::safe_getalloccount();
		{
			let config = WsConfig::default();
			let mut ws = WsHandler::new(config).unwrap();
			ws.start().unwrap();
			ws.add_server(WsServerConfig {
				addr: [127, 0, 0, 1],
				port: 9999,
				backlog: 10,
			})
			.unwrap();
			ws.stop().unwrap();
		}
		assert_eq!(initial, crate::sys::safe_getalloccount());
	}
}
