#[no_mangle]
pub extern "C" fn real_main(_argc: i32, _argv: *const *const u8) -> i32 {
	let _ = unsafe { crate::sys::getalloccount() };
	let _ = unsafe { crate::sys::getpagesize() };
	0
}
