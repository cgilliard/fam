#[macro_export]
macro_rules! panic {
	($msg:expr) => {{
		use sys::{_exit, cstring_len, write};
		write(2, $msg.as_ptr(), cstring_len($msg.as_ptr()));
		_exit(-1);
		loop {}
	}};
}
