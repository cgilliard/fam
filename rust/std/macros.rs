#[macro_export]
macro_rules! exit {
	($msg:expr) => {{
		unsafe {
			use core::panic::Location;
			use std::util::u128_to_str;
			use sys::{_exit, write};

			write(2, "Panic:\n".as_ptr(), 7);
			#[cfg(not(mrustc))]
			{
				let location = Location::caller();
				let file = location.file();
				let mut buf = [0u8; 32];
				let len = u128_to_str(location.line() as u128, 0, &mut buf);
				write(2, file.as_ptr(), file.len());
				write(2, ":".as_ptr(), 1);
				write(2, buf.as_ptr(), len);
				write(2, "\n".as_ptr(), 1);
			}

			write(2, $msg.as_ptr(), $msg.len());
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

#[macro_export]
macro_rules! pages {
	($v:expr) => {{
		use sys::getpagesize;
		let size = unsafe { getpagesize() };
		if size > 0 && $v > 0 {
			1 + ($v as usize - 1 as usize) / size as usize
		} else {
			1
		}
	}};
}

#[macro_export]
macro_rules! page_size {
	() => {{
		use sys::getpagesize;
		let v = unsafe { getpagesize() } as usize;
		v
	}};
}

#[macro_export]
macro_rules! aadd {
	($a:expr, $v:expr) => {{
		use sys::atomic_fetch_add_u64;
		unsafe { atomic_fetch_add_u64($a, $v) }
	}};
}

#[macro_export]
macro_rules! asub {
	($a:expr, $v:expr) => {{
		use sys::atomic_fetch_sub_u64;
		unsafe { atomic_fetch_sub_u64($a, $v) }
	}};
}

#[macro_export]
macro_rules! aload {
	($a:expr) => {{
		use sys::atomic_load_u64;
		unsafe { atomic_load_u64($a) }
	}};
}

#[macro_export]
macro_rules! astore {
	($a:expr, $v:expr) => {{
		use sys::atomic_store_u64;
		unsafe { atomic_store_u64($a, $v) }
	}};
}

#[macro_export]
macro_rules! cas {
	($v:expr, $expect:expr, $desired:expr) => {{
		use sys::cas_release;
		unsafe { cas_release($v, $expect, $desired) }
	}};
}

#[macro_export]
macro_rules! sched_yield {
	() => {{
		use sys::sched_yield;
		unsafe {
			sched_yield();
		}
	}};
}

#[macro_export]
macro_rules! print {
	($s:expr) => {{
		use sys::write;
		unsafe {
			write(2, $s.as_ptr(), $s.len());
		}
	}};
}

#[macro_export]
macro_rules! println {
	($s:expr) => {{
		use sys::write;
		unsafe {
			write(2, $s.as_ptr(), $s.len());
			write(2, "\n".as_ptr(), 1);
		}
	}};
}

#[macro_export]
macro_rules! print_num {
	($n:expr) => {{
		use core::str::from_utf8_unchecked;
		use std::util::i128_to_str;
		use sys::write;
		let mut buf = [0u8; 32];
		let len = i128_to_str($n as i128, &mut buf);
		unsafe {
			write(2, from_utf8_unchecked(&buf).as_ptr(), len);
		}
	}};
}

#[macro_export]
macro_rules! getmicros {
	() => {{
		use sys::getmicros;
		unsafe { getmicros() }
	}};
}
