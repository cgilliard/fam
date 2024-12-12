use sys::write;

#[no_mangle]
pub extern "C" fn real_main(_argc: i32, _argv: *const *const u8) -> i32 {
	unsafe {
		write(2, "real_main\n".as_ptr(), 10);
	}
	0
}

#[cfg(not(test))]
mod panic_mod {
	use core::option::Option::Some;
	use core::panic::PanicInfo;
	use sys::{_exit, write};
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
