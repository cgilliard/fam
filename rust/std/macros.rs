macro_rules! pages {
	($v:expr) => {{
		#[allow(unused_unsafe)]
		use sys::getpagesize;
		let size = unsafe { getpagesize() };
		if size > 0 {
			1 + ($v as u64 - 1 as u64) / size as u64
		} else {
			0
		}
	}};
}

#[macro_export]
macro_rules! page_size {
	() => {{
		#[allow(unused_unsafe)]
		use sys::getpagesize;
		let v = unsafe { getpagesize() } as u64;
		v
	}};
}

#[macro_export]
macro_rules! aadd {
	($a:expr, $v:expr) => {{
		#[allow(unused_unsafe)]
		use sys::atomic_fetch_add_i64;
		unsafe { atomic_fetch_add_i64($a, $v) }
	}};
}

#[macro_export]
macro_rules! asub {
	($a:expr, $v:expr) => {{
		#[allow(unused_unsafe)]
		use sys::atomic_fetch_sub_i64;
		unsafe { atomic_fetch_sub_i64($a, $v) }
	}};
}
