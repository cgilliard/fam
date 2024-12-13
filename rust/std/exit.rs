#[macro_export]
macro_rules! exit {
	($msg:expr) => {{
		#[allow(unused_unsafe)]
		unsafe {
			use core::panic::Location;
			use std::util::u32_to_str;
			use sys::{_exit, cstring_len, write};

			#[cfg(not(mrustc))]
			{
				let location = Location::caller();
				let file = location.file();
				let line = u32_to_str(location.line());
				write(2, file.as_ptr(), cstring_len(file.as_ptr()));
				write(2, line.as_ptr(), cstring_len(line.as_ptr()));
			}

			write(2, $msg.as_ptr(), cstring_len($msg.as_ptr()));
			write(2, "\n\0".as_ptr(), 1);
			_exit(-1);
			loop {}
		}
	}};
}

#[macro_export]
macro_rules! panic {
	($s:expr) => {{
		exit!($s);
	}};
}
