use prelude::*;

#[no_mangle]
pub extern "C" fn real_main(_argc: i32, _argv: *const *const u8) -> i32 {
	println!("real main {}", _argc);
	0
}
