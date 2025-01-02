#[macro_export]
macro_rules! rc {
	($v:expr) => {{
		match Rc::new($v) {
			Ok(v) => match v.clone() {
				Ok(v_clone) => Ok((v, v_clone)),
				Err(e) => Err(e),
			},
			Err(e) => Err(e),
		}
	}};
}

#[macro_export]
macro_rules! lock_pair {
	() => {{
		match lock_box!() {
			Ok(lock1) => match lock1.clone() {
				Ok(lock2) => Ok((lock1, lock2)),
				Err(e) => Err(e),
			},
			Err(e) => Err(e),
		}
	}};
}

#[macro_export]
macro_rules! writeb {
        ($f:expr, $fmt:expr) => {{
            writeb!($f, "{}", $fmt)
        }};
        ($f:expr, $fmt:expr, $($t:expr),*) => {{
            let mut err = err!(Unknown);
            match String::new($fmt) {
                Ok(fmt) => {
                    let mut cur = 0;
                    $(
                        match fmt.findn("{}", cur) {
                            Some(index) => {
                                match fmt.substring( cur, cur + index) {
                                    Ok(s) => {
                                        let s = s.to_str();
                                        match $f.write_str(s, s.len()) {
                                            Ok(_) => {},
                                            Err(e) => err = e,
                                        }
                                        cur += index + 2;
                                    }
                                    Err(e) => err = e,
                                }
                            },
                            None => {
                            },
                        }
                        match $t.format(&mut $f) {
                            Ok(_) => {},
                            Err(e) => err = e,
                        }
                    )*

                    match fmt.substring( cur, fmt.len()) {
                        Ok(s) => {
                            let s = s.to_str();
                            match $f.write_str(s, s.len()) {
                                Ok(_) =>{},
                                Err(e) => err = e,
                            }
                        }
                        Err(e) => err = e,
                    }
                }
                Err(e) => err = e,
            }

            if err.kind == ErrorKind::Unknown {
                Ok(())
            } else {
                Err(err)
            }
        }};
}

#[macro_export]
macro_rules! format {
        ($fmt:expr) => {{
                format!("{}", $fmt)
        }};
        ($fmt:expr, $($t:expr),*) => {{
                let mut formatter = Formatter::new();
                match writeb!(formatter, $fmt, $($t),*) {
                    Ok(_) => String::new(formatter.as_str()),
                    Err(e) => Err(e)
                }
        }};
}

#[macro_export]
macro_rules! exit {
        ($fmt:expr) => {{
                exit!("{}", $fmt);
        }};
        ($fmt:expr,  $($t:expr),*) => {{
                        use sys::safe_exit;

                        print!("Panic[@{}:{}]: ", file!(), line!());
                        println!($fmt, $($t),*);
                        safe_exit(-1);
                        loop {}
        }};
}

#[macro_export]
macro_rules! panic {
        ($fmt:expr) => {{
                exit!("{}", $fmt);
        }};
        ($fmt:expr,  $($t:expr),*) => {{
                exit!($fmt, $($t),*);
        }};
}

#[macro_export]
macro_rules! println {
    ($fmt:expr) => {{
            crate::sys::safe_write(2, $fmt.as_ptr(), $fmt.len());
            crate::sys::safe_write(2, "\n".as_ptr(), 1);
    }};
    ($fmt:expr, $($t:expr),*) => {{
        match format!($fmt, $($t),*) {
            Ok(line) => {
                crate::sys::safe_write(2, line.to_str().as_ptr(), line.len());
                crate::sys::safe_write(2, "\n".as_ptr(), 1);
            },
            Err(_e) => {},
        }
    }};
}

#[macro_export]
macro_rules! print {
    ($fmt:expr) => {{
        unsafe { crate::sys::write(2, $fmt.as_ptr(), $fmt.len()); }
    }};
    ($fmt:expr, $($t:expr),*) => {{
        match format!($fmt, $($t),*) {
            Ok(line) => {
                unsafe { crate::sys::write(2, line.to_str().as_ptr(), line.len()); }
            },
            Err(_e) => {},
        }
    }};
}

#[macro_export]
macro_rules! err {
	($kind:expr) => {{
		Error::new($kind, line!(), file!())
	}};
}

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
		safe_cas_release($v, $expect, $desired)
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
macro_rules! getmicros {
	() => {{
		use sys::safe_getmicros;
		safe_getmicros()
	}};
}

#[macro_export]
macro_rules! vec {
                ($($elem:expr),*) => {
                    #[allow(unused_mut)]
                    {
                                let mut vec = Vec::new();
                                let mut err: Error = err!(Unknown);
                                $(
                                        if err.kind == ErrorKind::Unknown {
                                                match vec.push($elem) {
                                                        Ok(_) => {},
                                                        Err(e) => err = e,
                                                }
                                        }
                                )*
                                if err.kind != ErrorKind::Unknown {
                                        Err(err)
                                } else {
                                        Ok(vec)
                                }
                    }
                };
}

#[macro_export]
macro_rules! lock {
	() => {{
		use core::cell::UnsafeCell;
		Lock {
			state: UnsafeCell::new(0),
		}
	}};
}

#[macro_export]
macro_rules! lock_box {
	() => {{
		LockBox::new()
	}};
}
