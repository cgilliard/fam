#[allow(dead_code)]
extern "C" {
	// allocation
	pub fn alloc(size: usize) -> *const u8;
	pub fn release(ptr: *const u8);
	pub fn resize(ptr: *const u8, size: usize) -> *const u8;

	// sys
	pub fn write(fd: i32, buf: *const u8, len: usize) -> i64;
	pub fn ptr_add(p: *mut u8, v: i64);
	pub fn exit(code: i32);
	pub fn getalloccount() -> usize;

	// misc
	pub fn sleep_millis(millis: u64) -> i32;
	pub fn rand_bytes(data: *mut u8, len: usize) -> i32;
	pub fn f64_to_str(d: f64, buf: *mut u8, capacity: u64) -> i32;

	// atomic
	pub fn atomic_store_u64(ptr: *mut u64, value: u64);
	pub fn atomic_load_u64(ptr: *const u64) -> u64;
	pub fn atomic_fetch_add_u64(ptr: *mut u64, value: u64) -> u64;
	pub fn atomic_fetch_sub_u64(ptr: *mut u64, value: u64) -> u64;
	pub fn cas_release(ptr: *mut u64, expect: *const u64, desired: u64) -> bool;

	// AES
	pub fn AES_ctx_size() -> usize;
	pub fn AES_init_ctx_iv(ctx: *mut u8, key: *const u8, iv: *const u8);
	pub fn AES_ctx_set_iv(ctx: *mut u8, iv: *const u8);
	pub fn AES_CTR_xcrypt_buffer(ctx: *mut u8, buf: *mut u8, len: u64);

	// cpsrng
	pub fn cpsrng_context_create() -> *mut u8;
	pub fn cpsrng_context_destroy(ctx: *mut u8);
	pub fn cpsrng_rand_bytes(ctx: *mut u8, v: *mut u8, len: usize);

	// aggsig
	pub fn secp256k1_aggsig_sign_single(
		cx: *const u8,
		sig: *mut u8,
		msg32: *const u8,
		seckey32: *const u8,
		secnonce32: *const u8,
		extra32: *const u8,
		pubnonce_for_e: *const u8,
		pubnonce_total: *const u8,
		pubkey_for_e: *const u8,
		seed32: *const u8,
	) -> i32;
	pub fn secp256k1_aggsig_verify_single(
		cx: *const u8,
		sig: *const u8,
		msg32: *const u8,
		pubnonce: *const u8,
		pk: *const u8,
		pk_total: *const u8,
		extra_pubkey: *const u8,
		is_partial: i32,
	) -> i32;
	pub fn secp256k1_context_create(flags: u32) -> *mut u8;
	pub fn secp256k1_context_destroy(ctx: *mut u8);
}

#[cfg(test)]
mod test {
	use super::*;
	#[test]
	fn test_aes1() {
		let mut buffer = [0u8; 1024];
		let ctx = &mut buffer as *mut u8;

		let key: [u8; 32] = [
			0x60, 0x3d, 0xeb, 0x10, 0x15, 0xca, 0x71, 0xbe, 0x2b, 0x73, 0xae, 0xf0, 0x85, 0x7d,
			0x77, 0x81, 0x1f, 0x35, 0x2c, 0x07, 0x3b, 0x61, 0x08, 0xd7, 0x2d, 0x98, 0x10, 0xa3,
			0x09, 0x14, 0xdf, 0xf4,
		];
		let mut input: [u8; 64] = [
			0x60, 0x1e, 0xc3, 0x13, 0x77, 0x57, 0x89, 0xa5, 0xb7, 0xa7, 0xf5, 0x04, 0xbb, 0xf3,
			0xd2, 0x28, 0xf4, 0x43, 0xe3, 0xca, 0x4d, 0x62, 0xb5, 0x9a, 0xca, 0x84, 0xe9, 0x90,
			0xca, 0xca, 0xf5, 0xc5, 0x2b, 0x09, 0x30, 0xda, 0xa2, 0x3d, 0xe9, 0x4c, 0xe8, 0x70,
			0x17, 0xba, 0x2d, 0x84, 0x98, 0x8d, 0xdf, 0xc9, 0xc5, 0x8d, 0xb6, 0x7a, 0xad, 0xa6,
			0x13, 0xc2, 0xdd, 0x08, 0x45, 0x79, 0x41, 0xa6,
		];
		let iv: [u8; 16] = [
			0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd,
			0xfe, 0xff,
		];
		let expected_output: [u8; 64] = [
			0x6b, 0xc1, 0xbe, 0xe2, 0x2e, 0x40, 0x9f, 0x96, 0xe9, 0x3d, 0x7e, 0x11, 0x73, 0x93,
			0x17, 0x2a, 0xae, 0x2d, 0x8a, 0x57, 0x1e, 0x03, 0xac, 0x9c, 0x9e, 0xb7, 0x6f, 0xac,
			0x45, 0xaf, 0x8e, 0x51, 0x30, 0xc8, 0x1c, 0x46, 0xa3, 0x5c, 0xe4, 0x11, 0xe5, 0xfb,
			0xc1, 0x19, 0x1a, 0x0a, 0x52, 0xef, 0xf6, 0x9f, 0x24, 0x45, 0xdf, 0x4f, 0x9b, 0x17,
			0xad, 0x2b, 0x41, 0x7b, 0xe6, 0x6c, 0x37, 0x10,
		];

		unsafe {
			AES_init_ctx_iv(ctx, &key as *const u8, &iv as *const u8);
			AES_CTR_xcrypt_buffer(ctx, &mut input as *mut u8, 64);
		}
		assert_eq!(input, expected_output, "AES CTR mode encryption failed!");
	}

	#[test]
	fn test_signature1() {
		unsafe {
			let mut sig = [0u8; 64];
			let msg = [9u8; 32];
			let seckey32 = [1u8; 32];
			let secnonce32 = [2u8; 32];
			let extra32 = [3u8; 32];
			let pubnonce_for_e = [4u8; 32];
			let pubnonce_total = [5u8; 32];
			let pubkey_for_e = [6u8; 32];
			let seed32 = [7u8; 32];

			let ctx = secp256k1_context_create(
				crate::constants::SECP256K1_START_SIGN | crate::constants::SECP256K1_START_SIGN,
			);

			secp256k1_aggsig_sign_single(
				ctx,
				&mut sig as *mut u8,
				&msg as *const u8,
				&seckey32 as *const u8,
				&secnonce32 as *const u8,
				&extra32 as *const u8,
				&pubnonce_for_e as *const u8,
				&pubnonce_total as *const u8,
				&pubkey_for_e as *const u8,
				&seed32 as *const u8,
			);

			secp256k1_context_destroy(ctx);
		}
	}
}
