/*#[macro_export]
macro_rules! println {
	($($arg:tt)*) => {{
		use util::write::Stdout;
		let _ = write!(Stdout, $($arg)*.to_string());
		let _ = write!(Stdout, "\n");
	}};
}
*/

pub trait Display {
	fn write(&self, buffer: &mut [u8]) -> usize;
}

/*
impl Display for i32 {
	fn write(&self, buffer: &mut [u8]) -> usize {
		let mut n = 0;
		let s = snprintf(&mut buffer[n..], *self);
		n += s.len();
		n
	}
}
*/

/*
#[macro_export]
macro_rules! println {
	($($arg:expr),*) => {{
		extern "C" {
			fn printf(format: *const u8, ...) -> i32;
		}
		$(
			unsafe {
				match $arg {
					 i32 => { printf("%i\0".as_ptr(), $arg); },
					 str => { printf("%s\0".as_ptr(), $arg); },
					_ => { printf("<?>".as_ptr()); }, // Add .as_ptr()
				}
			}
		)*
		unsafe {
			printf("\n".as_ptr()); // Add .as_ptr()
		}
	}};
}*/
