macro_rules! pages {
	($v:expr) => {{
		use sys::getpagesize;
		let size = unsafe { getpagesize() };
		if size > 0 {
			1 + ($v as usize - 1 as usize) / size as usize
		} else {
			0
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
