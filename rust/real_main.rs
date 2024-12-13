use crate::sys::write;
use exit;
use sys::cstring_len;

#[no_mangle]
pub extern "C" fn real_main(argc: i32, argv: *const *const u8) -> i32 {
	let mut print_len = 10;
	if argc > 0 {
		unsafe {
			let arg_ptr = *argv.offset(0);
			let len = cstring_len(arg_ptr);
			let mut buf = [0u8; 128];
			for i in 0..len {
				if i < 128 {
					buf[i] = *arg_ptr.add(i);
				}
			}
			if buf[0] == b't' && buf[1] == b'e' && buf[2] == b's' && buf[3] == b't' {
				print_len = 0;
			}
			if argc > 5 {
				exit!("argc > 5");
			}
		}
	} else {
		print_len = 0;
	}
	unsafe {
		write(2, "real_main\n".as_ptr(), print_len);
	}
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

	#[test]
	fn test_real_main() {
		let arg_silent = b"test\0";
		let argv: [*const u8; 2] = [arg_silent.as_ptr(), null()];
		assert_eq!(real_main(1, argv.as_ptr()), 0);
		assert_eq!(real_main(0, null()), 0);
	}
}
