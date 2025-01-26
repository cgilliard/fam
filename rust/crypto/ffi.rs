#![allow(dead_code)]

pub const SECP256K1_CONTEXT_NONE: u32 = 1;

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

	// SECP256k1
	pub fn secp256k1_context_create(flags: u32) -> *mut u8;
	pub fn secp256k1_context_destroy(ctx: *mut u8);
	pub fn secp256k1_context_randomize(ctx: *mut u8, seed: *const u8) -> i32;
	pub fn secp256k1_keypair_create(ctx: *mut u8, keypair: *mut u8, seckey: *const u8) -> i32;
	pub fn secp256k1_ec_seckey_verify(ctx: *mut u8, seckey: *const u8) -> i32;
	pub fn secp256k1_keypair_xonly_pub(
		ctx: *mut u8,
		xonly_pubkey: *mut u8,
		pk_parity: *mut i32,
		keypair: *const u8,
	) -> i32;
	pub fn secp256k1_xonly_pubkey_serialize(
		ctx: *mut u8,
		output: *mut u8,
		xonly_pubkey: *const u8,
	) -> i32;
	pub fn secp256k1_xonly_pubkey_parse(
		ctx: *mut u8,
		xonly_pubkey: *mut u8,
		input: *const u8,
		len: usize,
	) -> i32;
	pub fn secp256k1_ec_seckey_negate(ctx: *const u8, seckey: *mut u8) -> i32;
}
