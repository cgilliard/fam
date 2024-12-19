use sys::getmicros;

#[no_mangle]
pub extern "C" fn real_main(_argc: i32, _argv: *const *const u8) -> i32 {
	let _x = unsafe { getmicros() };
	0
}

#[allow(unexpected_cfgs)]
#[cfg(not(test))]
mod panic_mod {
	use crate::sys::{_exit, write};
	use core::option::Option::Some;
	use core::panic::PanicInfo;
	#[panic_handler]
	fn panic_handler(info: &PanicInfo) -> ! {
		#[cfg(not(mrustc))]
		{
			let panic_msg: &str = match info.message().as_str() {
				Some(x) => x,
				_ => "",
			};
			unsafe {
				write(2, "Panic:\n".as_ptr(), 7);
				write(2, panic_msg.as_ptr(), panic_msg.len() as usize);
				_exit(-1);
			}
		}
		#[cfg(mrustc)]
		{
			unsafe {
				write(2, "panic!\n".as_ptr(), 6);
				_exit(-1);
			}
		}
		loop {}
	}
}

#[cfg(test)]
mod test {
	use core::ptr::null;
	use real_main::real_main;
	use sys::write;

	#[test]
	fn test_real_main() {
		assert_eq!(real_main(0, null()), 0);
		unsafe {
			write(2, "".as_ptr(), 0);
		}
	}
}
