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
            unmap(ptr, pages);
		9		
       }

    }
}

