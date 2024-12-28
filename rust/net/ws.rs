use core::mem::size_of;
use core::ops::FnMut;
use core::ptr::null_mut;
use core::slice::from_raw_parts;
use core::str::from_utf8_unchecked;
use prelude::*;
use sys::{
	alloc, release, socket_accept, socket_close, socket_event_handle, socket_event_size, socket_fd,
	socket_handle_size, socket_listen, socket_multiplex_handle_size, socket_multiplex_init,
	socket_multiplex_register, socket_multiplex_wait, socket_recv, socket_send,
};

const REG_READ_FLAG: i32 = 0x1;
const REG_WRITE_FLAG: i32 = 0x1 << 1;

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
}

struct ConnectionInner {
	read: Vec<u8>,
	write: Vec<u8>,
	id: u64,
	handle: i32,
	lock: Lock,
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
		murmur3_32_of_slice(slice, MURMUR_SEED) as usize
	}
}

impl PartialEq for Handle {
	fn eq(&self, other: &Handle) -> bool {
		self.inner.handle == other.inner.handle
	}
}

impl Hash for Handle {
	fn hash(&self) -> usize {
		let slice = unsafe {
			from_raw_parts(
				&self.inner.handle as *const i32 as *const u8,
				size_of::<i32>(),
			)
		};
		murmur3_32_of_slice(slice, MURMUR_SEED) as usize
	}
}

struct WsContext {
	connections: Hashtable<Connection>,
	handles: Hashtable<Handle>,
}

impl WsContext {
	fn new() -> Result<Self, Error> {
		let connections = Hashtable::new(1024).unwrap();
		let handles = Hashtable::new(1024).unwrap();
		Ok(Self {
			connections,
			handles,
		})
	}
}

impl WsServer {
	pub fn new(config: WsConfig) -> Result<Self, Error> {
		let port = config.port;
		let handle = null_mut();
		Ok(Self {
			config,
			port,
			handle,
		})
	}

	pub fn register_handler(
		&mut self,
		path: &str,
		handler: Box<dyn FnMut(WsMessage, WsHandle) -> Result<(), Error>>,
	) -> Result<(), Error> {
		Ok(())
	}

	pub fn start(&mut self) -> Result<(), Error> {
		unsafe {
			let handle = alloc(socket_handle_size());
			let port = socket_listen(
				handle,
				self.config.addr.as_ptr(),
				self.config.port,
				self.config.backlog,
			);
			self.port = port as u16;
			self.handle = handle;
			match Self::start_threads(&self.config, handle) {
				Ok(_) => Ok(()),
				Err(e) => Err(e),
			}
		}
	}

	pub fn port(&self) -> u16 {
		self.port
	}

	pub fn stop(&mut self) {}

	unsafe fn read_proc(ehandle: *mut u8) {
		let buf: [u8; 512] = [0u8; 512];
		let len = socket_recv(ehandle, buf.as_ptr(), 512);

		let data = from_utf8_unchecked(from_raw_parts(buf.as_ptr(), len as usize));
		if len == 0 {
			socket_close(ehandle);
			println!("close {}", socket_fd(ehandle));
		} else {
			print!("msg[{}]={}", socket_fd(ehandle), data);
			socket_send(ehandle, buf.as_ptr(), len as usize);
		}
	}

	unsafe fn event_proc(
		multiplex: *mut u8,
		server: *mut u8,
		config: &WsConfig,
		events: *mut u8,
		count: i32,
		itt: &mut Rc<u64>,
		tid: usize,
	) {
		for i in 0..count {
			let mut ehandle = [0u8; 4];
			let ehandle: *mut u8 = &mut ehandle as *mut u8;
			socket_event_handle(
				ehandle,
				events.add(i as usize * socket_event_size() as usize),
			);
			if socket_fd(ehandle) == socket_fd(server) {
				let cur = aload!(&**itt);
				if cur as usize % config.threads == tid {
					let mut nhandle = [0u8; 4];
					let nhandle: *mut u8 = &mut nhandle as *mut u8;
					socket_accept(ehandle, nhandle);
					socket_multiplex_register(multiplex, nhandle, REG_READ_FLAG);

					aadd!(&mut **itt, 1);
					println!(
						"Thread[{}] accepted connection: {}",
						tid,
						socket_fd(nhandle)
					);
				}
			} else {
				Self::read_proc(ehandle);
			}
		}
	}

	fn start_threads(config: &WsConfig, server: *mut u8) -> Result<(), Error> {
		let itt: Rc<u64> = match Rc::new(0) {
			Ok(itt) => itt,
			Err(e) => return Err(e),
		};
		for tid in 0..config.threads {
			let mut context = match WsContext::new() {
				Ok(context) => context,
				Err(e) => exit!("Could not create ws context: {}", e),
			};
			let mut itt = match itt.clone() {
				Ok(itt) => itt,
				Err(e) => return Err(e),
			};
			let _ = spawnj(move || {
				let events = unsafe { alloc(socket_event_size() * config.max_events as usize) };
				let multiplex = unsafe { alloc(socket_multiplex_handle_size()) };
				if unsafe { socket_multiplex_init(multiplex) } < 0 {
					exit!("multiplex init");
				}
				if unsafe { socket_multiplex_register(multiplex, server, REG_READ_FLAG) } < 0 {
					exit!("multiplex reg");
				}
				loop {
					let count =
						unsafe { socket_multiplex_wait(multiplex, events, config.max_events) };
					unsafe {
						Self::event_proc(multiplex, server, config, events, count, &mut itt, tid);
					}
				}
			});
		}

		Ok(())
	}
}

#[cfg(test)]
mod test {
	use super::*;
	#[test]
	fn test_ws1() {
		let config = WsConfig {
			threads: 4,
			port: 9999,
			..Default::default()
		};
		//let mut ws = WsServer::new(config).unwrap();
		//assert!(ws.start().is_ok());
		//println!("port={}", ws.port());

		//		park();
	}
}
