#![allow(dead_code)]

pub const SECP256K1_CONTEXT_NONE: u32 = 1;

pub const GENERATOR_H: [u8; 64] = [
	0x50, 0x92, 0x9b, 0x74, 0xc1, 0xa0, 0x49, 0x54, 0xb7, 0x8b, 0x4b, 0x60, 0x35, 0xe9, 0x7a, 0x5e,
	0x07, 0x8a, 0x5a, 0x0f, 0x28, 0xec, 0x96, 0xd5, 0x47, 0xbf, 0xee, 0x9a, 0xce, 0x80, 0x3a, 0xc0,
	0x31, 0xd3, 0xc6, 0x86, 0x39, 0x73, 0x92, 0x6e, 0x04, 0x9e, 0x63, 0x7c, 0xb1, 0xb5, 0xf4, 0x0a,
	0x36, 0xda, 0xc2, 0x8a, 0xf1, 0x76, 0x69, 0x68, 0xc3, 0x0c, 0x23, 0x13, 0xf3, 0xa3, 0x89, 0x04,
];

pub const GENERATOR_G: [u8; 64] = [
	0x79, 0xbe, 0x66, 0x7e, 0xf9, 0xdc, 0xbb, 0xac, 0x55, 0xa0, 0x62, 0x95, 0xce, 0x87, 0x0b, 0x07,
	0x02, 0x9b, 0xfc, 0xdb, 0x2d, 0xce, 0x28, 0xd9, 0x59, 0xf2, 0x81, 0x5b, 0x16, 0xf8, 0x17, 0x98,
	0x48, 0x3a, 0xda, 0x77, 0x26, 0xa3, 0xc4, 0x65, 0x5d, 0xa4, 0xfb, 0xfc, 0x0e, 0x11, 0x08, 0xa8,
	0xfd, 0x17, 0xb4, 0x48, 0xa6, 0x85, 0x54, 0x19, 0x9c, 0x47, 0xd0, 0x8f, 0xfb, 0x10, 0xd4, 0xb8,
];

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
	) -> i32;
	pub fn secp256k1_ec_seckey_negate(ctx: *mut u8, seckey: *mut u8) -> i32;
	pub fn secp256k1_pedersen_commit(
		ctx: *mut u8,
		secp256k1_pedersen_commitment: *mut u8,
		blind: *const u8,
		value: u64,
		gen: *const u8,
	) -> i32;
	pub fn secp256k1_ec_pubkey_combine(
		ctx: *mut u8,
		out: *mut u8,
		ins: *const *const u8,
		n: usize,
	) -> i32;
	pub fn secp256k1_musig_nonce_gen(
		ctx: *mut u8,
		secnonce: *mut u8,
		pubnonce: *mut u8,
		session_id: *const u8,
		sk: *const u8,
		pubkey: *const u8,
		pubkeys: *const u8,
		additional_data: *const u8,
		secnonce2: *mut u8,
	) -> i32;
	pub fn secp256k1_musig_nonce_agg(
		ctx: *mut u8,
		aggnonce: *mut u8,
		pubnonces: *const *const u8,
		n: usize,
	) -> i32;
	pub fn secp256k1_musig_pubkey_agg(
		ctx: *mut u8,
		scratch: *mut u8,
		agg_pk: *mut u8,
		keyagg_cache: *mut u8,
		pubkeys: *const *const u8,
		n: usize,
	) -> i32;
	pub fn secp256k1_musig_nonce_process(
		ctx: *mut u8,
		session: *mut u8,
		aggnonce: *const u8,
		msg: *const u8,
		keyagg_cache: *const u8,
		additional_data: *const u8,
	) -> i32;

	pub fn secp256k1_musig_partial_sign(
		ctx: *mut u8,
		partial_sig: *mut u8,
		secnonce: *const u8,
		keypair: *const u8,
		keyagg_cache: *const u8,
		session: *const u8,
	) -> i32;
	pub fn secp256k1_musig_partial_sig_agg(
		ctx: *mut u8,
		final_sig: *mut u8,
		session: *const u8,
		partial_sigs: *const *const u8,
		n: usize,
	) -> i32;
	pub fn secp256k1_musig_partial_sig_verify(
		ctx: *mut u8,
		partial_sig: *const u8,
		pubnonce: *const u8,
		pubkey: *const u8,
		keyagg_cache: *const u8,
		session: *const u8,
	) -> i32;
}
