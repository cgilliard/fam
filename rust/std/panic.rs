#[macro_export]
macro_rules! panic {
	($s:expr) => {{
		use sys::{_exit, cstring_len, write};
		write(2, $s.as_ptr(), cstring_len($s.as_ptr()));
		write(2, "\n".as_ptr(), 1);
		_exit(-1);
		loop {}
	}};
}
