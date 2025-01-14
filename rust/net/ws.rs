#![allow(dead_code)]

use prelude::*;

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
	timeout_micros: i64,
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
	last: i64,
}

struct Connection {
	inner: Rc<ConnectionInner>,
}
