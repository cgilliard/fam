use sys::*;

#[no_mangle]
pub extern "C" fn real_main(_argc: i32, _argv: *const *const u8) -> i32 {
	unsafe {
		// Map memory for 2 pages
		let pages: u64 = 2;
		let ptr = map(pages);

		if ptr.is_null() {
			_exit(1);
		} else {
			let mut current_ptr = ptr;
			for i in 0..4096 * 2 {
				*current_ptr = (i % 26) as u8 + b'a'; // Write to memory
				current_ptr = current_ptr.add(1); // Move to the next byte
			}
			unmap(ptr, pages);
		}

		let _start = getnanos().to_u128();
		write(2, "test\n".as_ptr(), 5);

		os_sleep(2000);
		write(2, "test2\n".as_ptr(), 6);

		let size = getpagesize();
		size / 2048
	}
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
