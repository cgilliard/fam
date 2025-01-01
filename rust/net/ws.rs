use prelude::*;

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
	wakeup: [u8; 4],
	config: WsConfig,
	itt: u64,
	lock: Lock,
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
		match Hashtable::new(config.conn_hash_size) {
			Ok(handlers) => Ok(Self {
				runtime: None,
				mplexes: Vec::new(),
				handlers,
				wakeup: [0u8; 4],
				config,
				itt: 0,
				lock: lock!(),
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

	pub fn add_server(&mut self, config: WsServerConfig) -> Result<(), Error> {
		Err(err!(Todo))
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

		for tid in 0..self.state.config.threads {
			// SAFETY: unwrap ok on rc.clone
			//let state = self.state.clone().unwrap();
			/*
			runtime.execute(move || {
				println!("start thread {}: {}", tid, state.config.threads);
			});
						*/
		}

		self.state.runtime = Some(runtime);

		Ok(())
	}

	pub fn stop(&mut self) -> Result<(), Error> {
		match &mut self.state.runtime {
			Some(ref mut rt) => rt.stop(),
			None => Ok(()),
		}
		//	Err(err!(Todo))
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn test_ws1() {
		let config = WsConfig::default();
		let mut ws = WsHandler::new(config).unwrap();
		//ws.start().unwrap();
		//ws.stop().unwrap();
	}
}
