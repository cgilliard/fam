#![allow(dead_code)]

extern "C" {
	// AES
	pub fn AES_ctx_size() -> usize;
	pub fn AES_init_ctx_iv(ctx: *mut u8, key: *const u8, iv: *const u8);
	pub fn AES_ctx_set_iv(ctx: *mut u8, iv: *const u8);
	pub fn AES_CTR_xcrypt_buffer(ctx: *mut u8, buf: *mut u8, len: u64);

	// CPSRNG
	pub fn cpsrng_rand_bytes(v: *mut u8, len: usize);
	pub fn cpsrng_context_create() -> *mut u8;
	pub fn cpsrng_context_destroy(ctx: *mut u8);
	pub fn cpsrng_rand_bytes_ctx(ctx: *mut u8, v: *mut u8, len: usize);
}
