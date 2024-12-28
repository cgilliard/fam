#[macro_export]
macro_rules! aadd {
	($a:expr, $v:expr) => {{
		use sys::safe_atomic_fetch_add_u64;
		safe_atomic_fetch_add_u64($a, $v)
	}};
}

#[macro_export]
macro_rules! asub {
	($a:expr, $v:expr) => {{
		use sys::safe_atomic_fetch_sub_u64;
		safe_atomic_fetch_sub_u64($a, $v)
	}};
}

#[macro_export]
macro_rules! aload {
	($a:expr) => {{
		use sys::safe_atomic_load_u64;
		safe_atomic_load_u64($a)
	}};
}

#[macro_export]
macro_rules! astore {
	($a:expr, $v:expr) => {{
		use sys::safe_atomic_store_u64;
		safe_atomic_store_u64($a, $v)
	}};
}

#[macro_export]
macro_rules! cas {
	($v:expr, $expect:expr, $desired:expr) => {{
		use sys::safe_cas_release;
		sfae_cas_release($v, $expect, $desired)
	}};
}

#[macro_export]
macro_rules! sched_yield {
	() => {{
		use sys::safe_sched_yield;
		safe_sched_yield();
	}};
}

#[macro_export]
macro_rules! print_num {
	($n:expr) => {{
		use core::str::from_utf8_unchecked;
		use std::util::i128_to_str;
		use sys::safe_write;
		let mut buf = [0u8; 32];
		let len = i128_to_str($n as i128, &mut buf);
		safe_write(2, from_utf8_unchecked(&buf).as_ptr(), len);
	}};
}

#[macro_export]
macro_rules! ptr {
	($v:expr) => {{
		Ptr::new($v as *mut u8)
	}};
}

#[macro_export]
macro_rules! getmicros {
	() => {{
		use sys::safe_getmicros;
		safe_getmicros()
	}};
}
