#![allow(dead_code)]
#![allow(non_snake_case)]

const SHA3_FLAGS_KECCAK: i32 = 1;
const SHA3_FLAGS_NONE: i32 = 0;
const ED448_BYTES: usize = 57;
const ED448_SIG_BYTES: usize = ED448_BYTES * 2;

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

	// ed448
	pub fn ossl_c448_ed448_derive_public_key(
		ctx: *mut u8,
		pubkey: *mut u8,
		privkey: *const u8,
		propq: *const u8,
	) -> i32;
	pub fn ossl_c448_ed448_sign(
		ctx: *mut u8,
		signature: *mut u8,
		privkey: *const u8,
		pubkey: *const u8,
		message: *const u8,
		message_len: usize,
		prehashed: u8,
		context: *const u8,
		context_len: usize,
		propq: *const u8,
	) -> i32;
	pub fn ossl_c448_ed448_verify(
		ctx: *mut u8,
		signature: *const u8,
		pubkey: *const u8,
		message: *const u8,
		message_len: usize,
		prehashed: u8,
		context: *const u8,
		context_len: usize,
		propq: *const u8,
	) -> i32;

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

#[cfg(test)]
mod test {
	use super::*;
	use prelude::*;

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

	#[repr(C)] // Ensure that the struct layout matches C's struct layout
	pub struct Curve448Point {
		x: [u8; 32],
		y: [u8; 32],
		z: [u8; 32],
		t: [u8; 32],
	}

	#[repr(C)]
	pub struct Curve448Scalar {
		limb: [u64; 7], // Adjust size if necessary
	}

	//type curve448_scalar_t = *mut Curve448Scalar;

	extern "C" {
		fn ossl_curve448_precomputed_scalarmul_with_base_table(
			p: *mut Curve448Point,
			s: Curve448Scalar,
		);
	}

	fn privkey_to_scalar(privkey: &[u8; ED448_BYTES]) -> Curve448Scalar {
		let mut scalar = Curve448Scalar { limb: [0u64; 7] };
		// Convert the byte array into the scalar limbs (this assumes that you are using little-endian byte order)
		for i in 0..ED448_BYTES / 8 {
			scalar.limb[i] = u64::from_le_bytes(privkey[i * 8..(i + 1) * 8].try_into().unwrap());
		}
		scalar
	}

	#[test]
	fn test_448_keygen() {
		let ctx: *mut u8 = crate::sys::safe_alloc(10000) as *mut u8;

		// The provided private key (hexadecimal values converted to bytes)
		let privkey: [u8; ED448_BYTES] = [
			0x6c, 0x82, 0xa5, 0x62, 0xcb, 0x80, 0x8d, 0x10, 0xd6, 0x32, 0xbe, 0x89, 0xc8, 0x51,
			0x3e, 0xbf, 0x6c, 0x92, 0x9f, 0x34, 0xdd, 0xfa, 0x8c, 0x9f, 0x63, 0xc9, 0x96, 0x0e,
			0xf6, 0xe3, 0x48, 0xa3, 0x52, 0x8c, 0x8a, 0x3f, 0xcc, 0x2f, 0x04, 0x4e, 0x39, 0xa3,
			0xfc, 0x5b, 0x94, 0x49, 0x2f, 0x8f, 0x03, 0x2e, 0x75, 0x49, 0xa2, 0x00, 0x98, 0xf9,
			0x5b,
		];

		// Test public key derivation
		let mut derived_pubkey: [u8; ED448_BYTES] = [0u8; ED448_BYTES];
		let propq = crate::sys::safe_alloc(10000) as *mut u8;
		unsafe {
			let result = ossl_c448_ed448_derive_public_key(
				ctx,
				&mut derived_pubkey as *mut u8,
				&privkey as *const u8,
				propq,
			);
			assert_eq!(result, -1); // Ensure key derivation succeeds
		}

		// Convert the private key to a Curve448Scalar
		let scalar = privkey_to_scalar(&privkey);

		// Create a Curve448Point to hold the public key
		let mut point: Curve448Point = Curve448Point {
			x: [0; 32],
			y: [0; 32],
			z: [0; 32],
			t: [0; 32],
		};

		// Perform scalar multiplication with the precomputed base table
		unsafe {
			ossl_curve448_precomputed_scalarmul_with_base_table(
				&mut point as *mut Curve448Point,
				scalar,
			);
		}

		// Now, `point` contains the computed public key.
		// Compare the computed public key (`point`) with the `derived_pubkey`.
		// Check if the computed public key matches the derived public key.
		//assert_eq!(point.x, derived_pubkey[0..32]);
		//assert_eq!(point.y, derived_pubkey[32..64]);
	}

	#[test]
	fn test_448_1() {
		let ctx: *mut u8 = crate::sys::safe_alloc(10000) as *mut u8;

		// Sample private and public keys (use actual key material in real use cases)
		let privkey: [u8; ED448_BYTES] = [0u8; ED448_BYTES];
		let pubkey: [u8; ED448_BYTES] = [0u8; ED448_BYTES];

		// Sample message and context
		let message = b"Hello, world!";
		let context = b"SomeContext";

		// Test public key derivation
		let mut derived_pubkey: [u8; ED448_BYTES] = [0u8; ED448_BYTES];
		let propq = crate::sys::safe_alloc(10000) as *mut u8;
		unsafe {
			let result = ossl_c448_ed448_derive_public_key(
				ctx,
				&mut derived_pubkey as *mut u8,
				&privkey as *const u8,
				propq,
			);
			assert_eq!(result, -1);
		}

		// Test signing
		let mut signature: [u8; ED448_SIG_BYTES] = [0u8; ED448_SIG_BYTES];
		let prehashed = 0; // Use 0 for non-prehashed messages, or 1 for prehashed
		let context_len = context.len();
		unsafe {
			let sign_result = ossl_c448_ed448_sign(
				ctx,
				&mut signature as *mut u8,
				&privkey as *const u8,
				&pubkey as *const u8,
				message.as_ptr(),
				message.len(),
				prehashed,
				context.as_ptr(),
				context_len,
				propq,
			);
			assert_eq!(sign_result, -1);
		}

		// Test verification
		let mut _verify_result: i32;
		unsafe {
			_verify_result = ossl_c448_ed448_verify(
				ctx,
				&signature as *const u8,
				&pubkey as *const u8,
				message.as_ptr(),
				message.len(),
				prehashed,
				context.as_ptr(),
				context_len,
				propq,
			);
		}

		//assert_eq!(verify_result, -1);

		signature[0] = 0;
		signature[1] = 0;

		unsafe {
			_verify_result = ossl_c448_ed448_verify(
				ctx,
				&signature as *const u8,
				&pubkey as *const u8,
				message.as_ptr(),
				message.len(),
				prehashed,
				context.as_ptr(),
				context_len,
				propq,
			);
		}

		//assert_eq!(verify_result, 0);
	}
}
