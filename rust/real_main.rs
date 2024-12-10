use base::sys::map;
use base::sys::unmap;

#[no_mangle]
pub extern "C" fn real_main(argc: i32, argv: *const *const u8) -> i32 {
	unsafe {
		// Map memory for 2 pages
		let pages: u64 = 2;
		let ptr = map(pages);

		if ptr.is_null() {
			-1
		} else {
			let mut current_ptr = ptr;
			for i in 0..4096 * 2 {
				*current_ptr = (i % 26) as u8 + b'a'; // Write to memory
				current_ptr = current_ptr.add(1); // Move to the next byte
			}
			unmap(ptr, pages);
			9
		}
	}
}
