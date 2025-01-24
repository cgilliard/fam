#![allow(dead_code)]
#![allow(non_snake_case)]

mod sha2;
pub mod sha3;

pub const SHA3_FLAGS_KECCAK: i32 = 1;
pub const SHA3_FLAGS_NONE: i32 = 0;

pub const SECP256K1_CONTEXT_NONE: u32 = 1;

extern "C" {
	// AES
	pub fn AES_ctx_size() -> usize;
	pub fn AES_init_ctx_iv(ctx: *mut u8, key: *const u8, iv: *const u8);
	pub fn AES_ctx_set_iv(ctx: *mut u8, iv: *const u8);
	pub fn AES_CTR_xcrypt_buffer(ctx: *mut u8, buf: *mut u8, len: u64);

	// SHA3
	pub fn x_sha3_context_size() -> usize;
	pub fn sha3_Init256(ctx: *mut u8);
	pub fn sha3_Update(ctx: *mut u8, input: *const u8, len: usize);
	pub fn sha3_Finalize(ctx: *mut u8) -> *const u8;
	pub fn sha3_SetFlags(ctx: *mut u8, flags: i32);

	pub fn cpsrng_rand_bytes(v: *mut u8, len: usize);

	// SECP256k1
	pub fn secp256k1_context_create(flags: u32) -> *mut u8;
	pub fn secp256k1_context_destroy(ctx: *mut u8);
	pub fn secp256k1_context_randomize(ctx: *mut u8, seed: *const u8) -> i32;
	pub fn secp256k1_keypair_create(ctx: *mut u8, keypair: *mut u8, seckey: *const u8) -> i32;
	pub fn secp256k1_ec_seckey_verify(ctx: *mut u8, seckey: *const u8) -> i32;
	pub fn secp256k1_ec_pubkey_combine(
		ctx: *mut u8,
		combined_pubkey: *mut u8,
		pubkeys: *const u8,
		n: usize,
	) -> i32;
	pub fn secp256k1_keypair_xonly_pub(
		ctx: *mut u8,
		xonly_pubkey: *mut u8,
		pk_parity: *mut i32,
		keypair: *const u8,
	) -> i32;
	pub fn secp256k1_keypair_pub(ctx: *mut u8, pubkey: *mut u8, keypair: *const u8) -> i32;
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
	pub fn secp256k1_schnorrsig_sign32(
		ctx: *mut u8,
		sig64: *mut u8,
		msg32: *const u8,
		keypair: *const u8,
		aux_rand: *const u8,
	) -> i32;

	pub fn secp256k1_musig_session_initialize(
		ctx: *mut u8,
		session: *mut u8,
		pubkeys: *const u8,
		n: usize,
		aux_rand: *const u8,
	) -> i32;

	pub fn secp256k1_ec_seckey_negate(ctx: *const u8, seckey: *mut u8) -> i32;

	pub fn secp256k1_musig_aggregate_signatures(
		ctx: *mut u8,
		sig64: *mut u8,
		session: *mut u8,
		partial_sigs: *const u8,
		n: usize,
	) -> i32;
	/*
			pub fn create_keypair_and_pk(ctx: *mut u8, ...) -> i32;
			pub fn secp256k1_musig_nonce_gen(ctx: *mut u8, ...) -> i32;
			pub fn secp256k1_musig_pubkey_agg(ctx: *mut u8, ...) -> i32;
			pub fn secp256k1_musig_nonce_agg(ctx: *mut u8, ...) -> i32;
			pub fn secp256k1_musig_nonce_process(ctx: *mut u8, ...) -> i32;
			pub fn secp256k1_musig_partial_sign(ctx: *mut u8, ...) -> i32;
			pub fn secp256k1_musig_partial_sig_verify(ctx: *mut u8, ...) -> i32;
			pub fn secp256k1_musig_partial_sig_agg(ctx: *mut u8, ...) -> i32;
			pub fn secp256k1_schnorrsig_verify(ctx: *mut u8,  ...) -> i32;
	*/
	/*
	pub fn create_keypair_and_pk(
		ctx: *mut u8,          // context
		keypair: *mut u8,      // output keypair
		pubkey: *mut u8,       // output public key
		secret_key: *const u8, // input secret key (32 bytes)
	) -> i32;
		*/

	pub fn secp256k1_musig_nonce_gen(
		ctx: *mut u8,               // context
		secnonce: *mut u8,          // output secret nonce
		pubnonce: *mut u8,          // output public nonce
		session_id: *const u8,      // session ID (32 bytes)
		sk: *const u8,              // secret key (32 bytes)
		pubkey: *const u8,          // public key
		pubkeys: *const u8,         // optional: list of other public keys
		additional_data: *const u8, // optional: additional data
		secnonce2: *mut u8,         // optional: secondary secret nonce
	) -> i32;

	pub fn secp256k1_musig_pubkey_agg(
		ctx: *mut u8,              // context
		scratch: *mut u8,          // scratch space for aggregation
		agg_pk: *mut u8,           // output aggregated public key
		keyagg_cache: *mut u8,     // cache for key aggregation
		pubkeys: *const *const u8, // array of public keys
		n: usize,                  // number of public keys to aggregate
	) -> i32;

	pub fn secp256k1_musig_nonce_agg(
		ctx: *mut u8,                // context
		aggnonce: *mut u8,           // output aggregated nonce
		pubnonces: *const *const u8, // array of public nonces
		n: usize,                    // number of nonces to aggregate
	) -> i32;

	pub fn secp256k1_musig_nonce_process(
		ctx: *mut u8,               // context
		session: *mut u8,           // session data
		aggnonce: *const u8,        // aggregated nonce
		msg: *const u8,             // message to sign (32 bytes)
		keyagg_cache: *const u8,    // key aggregation cache
		additional_data: *const u8, // optional additional data
	) -> i32;

	pub fn secp256k1_musig_partial_sign(
		ctx: *mut u8,            // context
		partial_sig: *mut u8,    // output partial signature
		secnonce: *const u8,     // secret nonce
		keypair: *const u8,      // keypair
		keyagg_cache: *const u8, // key aggregation cache
		session: *const u8,      // session data
	) -> i32;

	pub fn secp256k1_musig_partial_sig_verify(
		ctx: *mut u8,            // context
		partial_sig: *const u8,  // partial signature
		pubnonce: *const u8,     // public nonce
		pubkey: *const u8,       // public key
		keyagg_cache: *const u8, // key aggregation cache
		session: *const u8,      // session data
	) -> i32;

	pub fn secp256k1_musig_partial_sig_agg(
		ctx: *mut u8,                   // context
		final_sig: *mut u8,             // output final aggregated signature (64 bytes)
		session: *const u8,             // session data
		partial_sigs: *const *const u8, // array of partial signatures
		n: usize,                       // number of partial signatures
	) -> i32;

	pub fn secp256k1_schnorrsig_verify(
		ctx: *mut u8,   // context
		sig: *const u8, // signature (64 bytes)
		msg: *const u8, // message to verify (32 bytes)
		msg_len: usize, // length of the message
		pk: *const u8,  // public key
	) -> i32;

	pub fn secp256k1_keypair_negate(ctx: *mut u8, keypair: *mut u8);

	pub fn secp256k1_fe_is_odd(fe: *const u8) -> i32;
	pub fn secp256k1_fe_impl_set_b32_mod(fe: *mut u8, xonly_pubkey: *const u8);
	pub fn secp256k1_keypair_xonly_tweak_add(ctx: *mut u8, keypair: *mut u8, tweak: *const u8);

}

pub fn safe_cpsrng_rand_bytes(v: *mut u8, len: usize) {
	unsafe { cpsrng_rand_bytes(v, len) }
}

pub fn safe_AES_ctx_size() -> usize {
	unsafe { AES_ctx_size() }
}

pub fn safe_AES_init_ctx_iv(ctx: *mut u8, key: *const u8, iv: *const u8) {
	unsafe { AES_init_ctx_iv(ctx, key, iv) }
}

pub fn safe_AES_ctx_set_iv(ctx: *mut u8, iv: *const u8) {
	unsafe { AES_ctx_set_iv(ctx, iv) }
}

pub fn safe_AES_CTR_xcrypt_buffer(ctx: *mut u8, buf: *mut u8, len: u64) {
	unsafe { AES_CTR_xcrypt_buffer(ctx, buf, len) }
}

pub fn safe_sha3_context_size() -> usize {
	unsafe { x_sha3_context_size() }
}

pub fn safe_sha3_SetFlags(ctx: *mut u8, flags: i32) {
	unsafe { sha3_SetFlags(ctx, flags) }
}

pub fn safe_sha3_Init256(ctx: *mut u8) {
	unsafe { sha3_Init256(ctx) }
}

pub fn safe_sha3_Update(ctx: *mut u8, input: *const u8, len: usize) {
	unsafe { sha3_Update(ctx, input, len) }
}
pub fn safe_sha3_Finalize(ctx: *mut u8) -> *const u8 {
	unsafe { sha3_Finalize(ctx) }
}

pub fn safe_secp256k1_context_create(flags: u32) -> *mut u8 {
	unsafe {
		let ctx = secp256k1_context_create(flags);
		let mut r = [0u8; 32];
		cpsrng_rand_bytes(&mut r as *mut u8, 32);
		secp256k1_context_randomize(ctx, &r as *const u8);
		ctx
	}
}

pub fn safe_secp256k1_context_destroy(ctx: *mut u8) {
	unsafe { secp256k1_context_destroy(ctx) }
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn test_secp256k1_1() {
		let ctx = safe_secp256k1_context_create(SECP256K1_CONTEXT_NONE);
		assert!(!ctx.is_null());
		let mut seckey = [0u8; 32];
		unsafe {
			cpsrng_rand_bytes(&mut seckey as *mut u8, 32);
			assert!(secp256k1_ec_seckey_verify(ctx, &seckey as *const u8) == 1);
		}

		let mut keypair = [0u8; 96];
		let mut xonly_pubkey = [0u8; 64];
		unsafe {
			use core::ptr::null_mut;
			assert!(
				secp256k1_keypair_create(ctx, &mut keypair as *mut u8, &seckey as *const u8) != 0
			);
			assert!(
				secp256k1_keypair_xonly_pub(
					ctx,
					&mut xonly_pubkey as *mut u8,
					null_mut(),
					&keypair as *const u8,
				) == 1
			);
		}
		let mut sig64 = [0u8; 64];
		let msg32 = [8u8; 32];
		let mut r = [0u8; 32];
		unsafe {
			cpsrng_rand_bytes(&mut r as *mut u8, 32);
			assert!(
				secp256k1_schnorrsig_sign32(
					ctx,
					&mut sig64 as *mut u8,
					&msg32 as *const u8,
					&keypair as *const u8,
					&r as *const u8,
				) == 1
			);

			assert!(
				secp256k1_schnorrsig_verify(
					ctx,
					&sig64 as *const u8,
					&msg32 as *const u8,
					32,
					&xonly_pubkey as *const u8,
				) == 1
			);
		}

		safe_secp256k1_context_destroy(ctx);
	}

	#[test]
	fn test_musig_simple() {
		use core::ptr;
		// Create a context for secp256k1
		let ctx = safe_secp256k1_context_create(SECP256K1_CONTEXT_NONE);
		assert!(!ctx.is_null());

		// Generate secret keys for two participants
		let mut sk1 = [0u8; 32];
		let mut sk2 = [0u8; 32];
		unsafe {
			cpsrng_rand_bytes(&mut sk1 as *mut u8, 32);
			cpsrng_rand_bytes(&mut sk2 as *mut u8, 32);
		}

		// Create keypairs from the secret keys
		let mut keypair1 = [0u8; 1000];
		let mut keypair2 = [0u8; 1000];
		let mut pk1 = [0u8; 1000];
		let mut pk2 = [0u8; 1000];

		unsafe {
			assert!(secp256k1_ec_seckey_verify(ctx, &sk1 as *const u8) == 1);
			assert!(secp256k1_ec_seckey_verify(ctx, &sk2 as *const u8) == 1);

			// Create keypair
			assert!(
				secp256k1_keypair_create(ctx, &mut keypair1 as *mut u8, &sk1 as *const u8) == 1
			);

			let mut pk_parity = -1;
			assert!(
				secp256k1_keypair_xonly_pub(
					ctx,
					&mut pk1 as *mut u8,
					&mut pk_parity as *mut i32,
					&keypair1 as *const u8,
				) == 1
			);

			if pk_parity == 1 {
				secp256k1_ec_seckey_negate(ctx, &mut sk1 as *mut u8);
				assert!(
					secp256k1_keypair_create(ctx, &mut keypair1 as *mut u8, &sk1 as *const u8) == 1
				);

				assert!(
					secp256k1_keypair_xonly_pub(
						ctx,
						&mut pk1 as *mut u8,
						&mut pk_parity as *mut i32,
						&keypair1 as *const u8,
					) == 1
				);
				assert_eq!(pk_parity, 0);
			}

			assert!(
				secp256k1_keypair_create(ctx, &mut keypair2 as *mut u8, &sk2 as *const u8) == 1
			);

			let mut pk_parity = -1;
			assert!(
				secp256k1_keypair_xonly_pub(
					ctx,
					&mut pk2 as *mut u8,
					&mut pk_parity as *mut i32,
					&keypair2 as *const u8,
				) == 1
			);
			if pk_parity == 1 {
				secp256k1_ec_seckey_negate(ctx, &mut sk2 as *mut u8);
				assert!(
					secp256k1_keypair_create(ctx, &mut keypair2 as *mut u8, &sk2 as *const u8) == 1
				);

				assert!(
					secp256k1_keypair_xonly_pub(
						ctx,
						&mut pk2 as *mut u8,
						&mut pk_parity as *mut i32,
						&keypair2 as *const u8,
					) == 1
				);
				assert_eq!(pk_parity, 0);
			}
		}

		// Generate nonces and session ID
		let mut pubnonce1 = [0u8; 1000]; // Public nonces (132 bytes)
		let mut pubnonce2 = [0u8; 1000]; // second nonce
		let mut secnonce1 = [0u8; 1000]; // Secret nonces (132 bytes)
		let mut secnonce2 = [0u8; 1000];
		let mut session_id1 = [0u8; 1000]; // Session IDs (32 bytes)
		let mut session_id2 = [0u8; 1000];
		let mut aggnonce = [0u8; 1000]; // Aggregated nonce (132 bytes)

		unsafe {
			// Generate nonces for each participant
			cpsrng_rand_bytes(&mut session_id1 as *mut u8, 32);
			assert!(
				secp256k1_musig_nonce_gen(
					ctx,
					&mut secnonce1 as *mut u8,
					&mut pubnonce1 as *mut u8,
					&session_id1 as *const u8,
					&sk1 as *const u8,
					&pk1 as *const u8,
					ptr::null(),
					ptr::null(),
					ptr::null_mut()
				) == 1
			);

			cpsrng_rand_bytes(&mut session_id2 as *mut u8, 32);
			assert!(
				secp256k1_musig_nonce_gen(
					ctx,
					&mut secnonce2 as *mut u8,
					&mut pubnonce2 as *mut u8,
					&session_id2 as *const u8,
					&sk2 as *const u8,
					&pk2 as *const u8,
					ptr::null(),
					ptr::null(),
					ptr::null_mut()
				) == 1
			);

			let pubnonce_ptrs: [*const u8; 2] = [&pubnonce1 as *const u8, &pubnonce2 as *const u8];

			// Aggregate the public nonces
			assert!(
				secp256k1_musig_nonce_agg(ctx, &mut aggnonce as *mut u8, pubnonce_ptrs.as_ptr(), 2)
					== 1
			);
		}

		// Prepare for signing and verification
		let mut keyagg_cache = [0u8; 1000]; // Cache for key aggregation
		let mut agg_pk = [0u8; 1000]; // Aggregated public key
		let mut session = [0u8; 1000]; // Session state

		let pks_ptrs: [*const u8; 2] = [&pk1 as *const u8, &pk2 as *const u8];
		let msg = [37u8; 1000];

		unsafe {
			// Aggregate public keys
			assert!(
				secp256k1_musig_pubkey_agg(
					ctx,
					ptr::null_mut(),
					&mut agg_pk as *mut u8,
					&mut keyagg_cache as *mut u8,
					pks_ptrs.as_ptr(),
					2
				) == 1
			);

			// Process nonce aggregation
			assert!(
				secp256k1_musig_nonce_process(
					ctx,
					&mut session as *mut u8,
					&aggnonce as *const u8,
					&msg as *const u8,
					&keyagg_cache as *const u8,
					ptr::null()
				) == 1
			);
		}

		let mut partial_sig1 = [0u8; 1000];
		let mut partial_sig2 = [0u8; 1000];

		unsafe {
			assert!(
				secp256k1_musig_partial_sign(
					ctx,
					&mut partial_sig1 as *mut u8,
					&secnonce1 as *const u8,
					&keypair1 as *const u8,
					&keyagg_cache as *const u8,
					&session as *const u8
				) == 1
			);

			assert!(
				secp256k1_musig_partial_sig_verify(
					ctx,
					&partial_sig1 as *const u8,
					&pubnonce1 as *const u8,
					&pk1 as *const u8,
					&keyagg_cache as *const u8,
					&session as *const u8
				) == 1
			);

			assert!(
				secp256k1_musig_partial_sign(
					ctx,
					&mut partial_sig2 as *mut u8,
					&secnonce2 as *const u8,
					&keypair2 as *const u8,
					&keyagg_cache as *const u8,
					&session as *const u8
				) == 1
			);

			assert!(
				secp256k1_musig_partial_sig_verify(
					ctx,
					&partial_sig2 as *const u8,
					&pubnonce2 as *const u8,
					&pk2 as *const u8,
					&keyagg_cache as *const u8,
					&session as *const u8
				) == 1
			);

			// Aggregate partial signatures to get the final signature
			let mut final_sig = [0u8; 64];
			let partial_sig = [&partial_sig1 as *const u8, &partial_sig2 as *const u8];
			assert!(
				secp256k1_musig_partial_sig_agg(
					ctx,
					&mut final_sig as *mut u8,
					&session as *const u8,
					partial_sig.as_ptr(),
					2
				) == 1
			);

			// Verify the final Schnorr signature
			assert!(
				secp256k1_schnorrsig_verify(
					ctx,
					&final_sig as *const u8,
					&msg as *const u8,
					32,
					&agg_pk as *const u8
				) == 1
			);
		}

		// Clean up the context
		safe_secp256k1_context_destroy(ctx);
	}

	#[test]
	fn test_aes1() {
		let ctx = crate::sys::safe_alloc(safe_AES_ctx_size()) as *mut u8;

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

		safe_AES_init_ctx_iv(ctx, &key as *const u8, &iv as *const u8);
		safe_AES_CTR_xcrypt_buffer(ctx, &mut input as *mut u8, 64);
		assert_eq!(input, expected_output, "AES CTR mode encryption failed!");

		crate::sys::safe_release(ctx);
	}

	#[test]
	fn test_aes_rfc3686() {
		let ctx = crate::sys::safe_alloc(safe_AES_ctx_size()) as *mut u8;

		// Key
		let key: [u8; 32] = [
			0x60, 0x3d, 0xeb, 0x10, 0x15, 0xca, 0x71, 0xbe, 0x2b, 0x73, 0xae, 0xf0, 0x85, 0x7d,
			0x77, 0x81, 0x1f, 0x35, 0x2c, 0x07, 0x3b, 0x61, 0x08, 0xd7, 0x2d, 0x98, 0x10, 0xa3,
			0x09, 0x14, 0xdf, 0xf4,
		];

		// IV/Nonce
		let iv: [u8; 16] = [
			0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd,
			0xfe, 0xff,
		];

		// Input Plaintext
		let mut input: [u8; 64] = [
			0x6b, 0xc1, 0xbe, 0xe2, 0x2e, 0x40, 0x9f, 0x96, 0xe9, 0x3d, 0x7e, 0x11, 0x73, 0x93,
			0x17, 0x2a, 0xae, 0x2d, 0x8a, 0x57, 0x1e, 0x03, 0xac, 0x9c, 0x9e, 0xb7, 0x6f, 0xac,
			0x45, 0xaf, 0x8e, 0x51, 0x30, 0xc8, 0x1c, 0x46, 0xa3, 0x5c, 0xe4, 0x11, 0xe5, 0xfb,
			0xc1, 0x19, 0x1a, 0x0a, 0x52, 0xef, 0xf6, 0x9f, 0x24, 0x45, 0xdf, 0x4f, 0x9b, 0x17,
			0xad, 0x2b, 0x41, 0x7b, 0xe6, 0x6c, 0x37, 0x10,
		];

		// Expected Ciphertext
		let expected_output: [u8; 64] = [
			0x60, 0x1e, 0xc3, 0x13, 0x77, 0x57, 0x89, 0xa5, 0xb7, 0xa7, 0xf5, 0x04, 0xbb, 0xf3,
			0xd2, 0x28, 0xf4, 0x43, 0xe3, 0xca, 0x4d, 0x62, 0xb5, 0x9a, 0xca, 0x84, 0xe9, 0x90,
			0xca, 0xca, 0xf5, 0xc5, 0x2b, 0x09, 0x30, 0xda, 0xa2, 0x3d, 0xe9, 0x4c, 0xe8, 0x70,
			0x17, 0xba, 0x2d, 0x84, 0x98, 0x8d, 0xdf, 0xc9, 0xc5, 0x8d, 0xb6, 0x7a, 0xad, 0xa6,
			0x13, 0xc2, 0xdd, 0x08, 0x45, 0x79, 0x41, 0xa6,
		];

		// Initialize AES context with key and IV
		safe_AES_init_ctx_iv(ctx, &key as *const u8, &iv as *const u8);

		// Encrypt in CTR mode
		safe_AES_CTR_xcrypt_buffer(ctx, &mut input as *mut u8, 64);

		// Compare the output to the expected ciphertext
		assert_eq!(input, expected_output, "AES-256-CTR encryption failed!");

		crate::sys::safe_release(ctx);
	}

	#[test]
	fn test_sha3_256_keccak() {
		// Initialize the SHA3 context with the appropriate size for SHA3-256.
		let sz = safe_sha3_context_size();
		let ctx = crate::sys::safe_alloc(sz) as *mut u8;

		// Initialize SHA3-256 with Keccak flag.
		safe_sha3_Init256(ctx);
		safe_sha3_SetFlags(ctx, SHA3_FLAGS_KECCAK);

		// Prepare the input data (same as in the C test).
		let input_data: [u8; 100] = [
			0x43, 0x3c, 0x53, 0x03, 0x13, 0x16, 0x24, 0xc0, 0x02, 0x1d, 0x86, 0x8a, 0x30, 0x82,
			0x54, 0x75, 0xe8, 0xd0, 0xbd, 0x30, 0x52, 0xa0, 0x22, 0x18, 0x03, 0x98, 0xf4, 0xca,
			0x44, 0x23, 0xb9, 0x82, 0x14, 0xb6, 0xbe, 0xaa, 0xc2, 0x1c, 0x88, 0x07, 0xa2, 0xc3,
			0x3f, 0x8c, 0x93, 0xbd, 0x42, 0xb0, 0x92, 0xcc, 0x1b, 0x06, 0xce, 0xdf, 0x32, 0x24,
			0xd5, 0xed, 0x1e, 0xc2, 0x97, 0x84, 0x44, 0x4f, 0x22, 0xe0, 0x8a, 0x55, 0xaa, 0x58,
			0x54, 0x2b, 0x52, 0x4b, 0x02, 0xcd, 0x3d, 0x5d, 0x5f, 0x69, 0x07, 0xaf, 0xe7, 0x1c,
			0x5d, 0x74, 0x62, 0x22, 0x4a, 0x3f, 0x9d, 0x9e, 0x53, 0xe7, 0xe0, 0x84, 0x6d, 0xcb,
			0xb4, 0xce,
		];

		// Update the context with the input data.
		safe_sha3_Update(ctx, input_data.as_ptr(), input_data.len());

		// Finalize the SHA3-256 hash.
		let hash = safe_sha3_Finalize(ctx);

		// The expected output hash from the C test.
		let expected_hash: [u8; 32] = [
			0xce, 0x87, 0xa5, 0x17, 0x3b, 0xff, 0xd9, 0x23, 0x99, 0x22, 0x16, 0x58, 0xf8, 0x01,
			0xd4, 0x5c, 0x29, 0x4d, 0x90, 0x06, 0xee, 0x9f, 0x3f, 0x9d, 0x41, 0x9c, 0x8d, 0x42,
			0x77, 0x48, 0xdc, 0x41,
		];

		use core::slice::from_raw_parts;
		let hash_slice = unsafe { from_raw_parts(hash, 32) };

		// Assert that the hash matches the expected output.
		assert_eq!(
			hash_slice, expected_hash,
			"SHA3-256(433C...CE) doesn't match known answer (single buffer)"
		);

		// Release the context.
		crate::sys::safe_release(ctx);
	}
}
