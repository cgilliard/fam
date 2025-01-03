#![allow(dead_code)]
#![allow(unused_variables)]

const EAGAIN: i32 = -11;
const REG_READ_FLAG: i32 = 0x1;
const REG_WRITE_FLAG: i32 = 0x2;

use prelude::*;
use sys::{
	safe_alloc, safe_pipe, safe_release, safe_socket_clear_pipe, safe_socket_close,
	safe_socket_event_handle, safe_socket_event_size, safe_socket_handle_eq, safe_socket_listen,
	safe_socket_multiplex_init, safe_socket_multiplex_register, safe_socket_multiplex_wait,
	safe_socket_send,
};

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

struct WriteHandle {
	buffer: Vec<u8>,
	lock: Lock,
	handle: [u8; 4],
	mplex: [u8; 4],
	state: ConnectionState,
}

impl WriteHandle {
	fn new(handle: [u8; 4], mplex: [u8; 4]) -> Self {
		Self {
			buffer: Vec::new(),
			lock: lock!(),
			handle,
			mplex,
			state: ConnectionState::NeedHandshake,
		}
	}

	fn writeb(&mut self, msg: &[u8]) -> Result<(), Error> {
		let _l = self.lock.write();
		Err(err!(Todo))
	}

	fn write(&mut self, msg: &str) -> Result<(), Error> {
		self.writeb(msg.as_bytes())
	}
}

enum ConnectionType {
	Server,
	ServerConnection,
	ClientConnection,
}

struct ConnectionInner {
	ctype: ConnectionType,
	read: Vec<u8>,
	wh: WriteHandle,
}

struct Connection {
	inner: Rc<ConnectionInner>,
}

pub struct WsRequest<'a> {
	msg: WsMessage<'a>,
}
pub struct WsResponse {
	wh: WriteHandle,
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
	handlers: Hashtable<Handler>,
	wakeup: [u8; 8],
	config: WsConfig,
	itt: u64,
	lock: LockBox,
	halt: bool,
}

pub struct WsContext {
	state: Rc<State>,
	connections: Hashtable<Connection>,
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
				lock,
				halt: false,
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

		for mplex in &self.state.mplexes {
			if safe_socket_multiplex_register(
				mplex as *const u8,
				&server as *const u8,
				REG_READ_FLAG,
			) < 0
			{
				safe_socket_close(server_ptr);
				return Err(err!(MultiplexRegister));
			}
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
			let connections = match Hashtable::new(1024) {
				Ok(connections) => connections,
				Err(e) => return Err(e),
			};
			let mut mplex = [0u8; 4];

			if safe_socket_multiplex_init(&mut mplex as *mut u8) < 0 {
				return Err(err!(CreateFileDescriptor));
			}

			match state.mplexes.push(mplex) {
				Ok(_) => {}
				Err(e) => return Err(e),
			}

			if safe_socket_multiplex_register(
				&mplex as *const u8,
				&self.state.wakeup as *const u8,
				REG_READ_FLAG,
			) < 0
			{
				return Err(err!(MultiplexRegister));
			}

			let events =
				safe_alloc(safe_socket_event_size() * self.state.config.max_events as usize)
					as *mut u8;

			let mut ctx = WsContext {
				state,
				connections,
				tid,
				mplex,
				events,
			};

			let _ = runtime.execute(move || match Self::event_loop(&mut ctx) {
				Ok(_) => {}
				Err(e) => println!("FATAL: unexpected error in thread_loop: {}", e),
			});
		}

		self.state.runtime = Some(runtime);

		Ok(())
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

	fn event_loop(ctx: &mut WsContext) -> Result<(), Error> {
		let mut ehandle = [0u8; 4];
		let ehandle: *mut u8 = &mut ehandle as *mut u8;
		let wakeup = &ctx.state.wakeup as *const u8;
		let mut stop = false;

		while !stop {
			let count = safe_socket_multiplex_wait(
				&mut ctx.mplex as *mut u8,
				ctx.events,
				ctx.state.config.max_events,
			);
			{
				let _l = ctx.state.lock.write();
				println!("count[{}]={}", ctx.tid, count);
			}
			for i in 0..count {
				let evt = unsafe {
					ctx.events
						.add(i as usize * safe_socket_event_size() as usize)
				};
				safe_socket_event_handle(ehandle, evt);
				if safe_socket_handle_eq(ehandle, wakeup) {
					safe_socket_clear_pipe(ehandle);
					let _l = ctx.state.lock.read();
					if ctx.state.halt {
						stop = true;
						break;
					}
				} else {
				}
			}
		}

		safe_socket_close(&mut ctx.mplex as *mut u8);
		safe_release(ctx.events);

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
