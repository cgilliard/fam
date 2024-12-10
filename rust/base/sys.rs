extern "C" {
    pub fn map(pages: u64) -> *mut u8;	
    pub fn unmap(ptr: *mut u8, pages: u64);
}
