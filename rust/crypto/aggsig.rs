#[derive(Clone, Copy)]
pub struct AggSig([u8; 96]);

#[cfg(test)]
mod test {
	use super::*;
	use core::ptr::null;
	use crypto::context::Context;
	use crypto::ffi::{
		secp256k1_ec_pubkey_combine, secp256k1_keypair_create, secp256k1_keypair_pub,
		secp256k1_keypair_xonly_pub, secp256k1_schnorrsig_sign32, secp256k1_schnorrsig_verify,
		secp256k1_xonly_pubkey_from_pubkey,
	};
	use crypto::keys::PrivateKey;
	use prelude::*;

	#[test]
	fn test_agg_sig() {
		let msg = [9u8; 32];
		let msg_len = 32;

		let mut sig1 = [0u8; 64];
		let mut keypair1 = [0u8; 96];
		let mut xonly_pk1 = [0u8; 64];
		let mut pk_parity1 = -1;

		let mut sig2 = [0u8; 64];
		let mut keypair2 = [0u8; 96];
		let mut xonly_pk2 = [0u8; 64];
		let mut pk_parity2 = -1;

		let mut ctx = Context::new().unwrap();

		let seckey1 = PrivateKey::generate(&mut ctx).unwrap();
		let seckey2 = PrivateKey::generate(&mut ctx).unwrap();

		unsafe {
			assert_eq!(
				secp256k1_keypair_create(
					ctx.secp(),
					&mut keypair1 as *mut u8,
					seckey1.as_ref().as_ptr(),
				),
				1
			);

			assert_eq!(
				secp256k1_keypair_create(
					ctx.secp(),
					&mut keypair2 as *mut u8,
					seckey2.as_ref().as_ptr(),
				),
				1
			);

			assert_eq!(
				secp256k1_schnorrsig_sign32(
					ctx.secp(),
					&mut sig1 as *mut u8,
					&msg as *const u8,
					&keypair1 as *const u8,
					null(),
				),
				1
			);

			assert_eq!(
				secp256k1_schnorrsig_sign32(
					ctx.secp(),
					&mut sig2 as *mut u8,
					&msg as *const u8,
					&keypair2 as *const u8,
					null(),
				),
				1
			);

			assert_eq!(
				secp256k1_keypair_xonly_pub(
					ctx.secp(),
					&mut xonly_pk1 as *mut u8,
					&mut pk_parity1 as *mut i32,
					&keypair1 as *const u8,
				),
				1
			);

			assert_eq!(
				secp256k1_keypair_xonly_pub(
					ctx.secp(),
					&mut xonly_pk2 as *mut u8,
					&mut pk_parity2 as *mut i32,
					&keypair2 as *const u8,
				),
				1
			);

			assert_eq!(
				secp256k1_schnorrsig_verify(
					ctx.secp(),
					&sig1 as *const u8,
					&msg as *const u8,
					msg_len,
					&xonly_pk1 as *const u8,
				),
				1
			);

			assert_eq!(
				secp256k1_schnorrsig_verify(
					ctx.secp(),
					&sig2 as *const u8,
					&msg as *const u8,
					msg_len,
					&xonly_pk2 as *const u8,
				),
				1
			);

			// Addtional outline to extend these partial signatures to sign and
			// validate an aggregated signature:
			// 1.) Add public keys using secp256k1_ec_pubkey_combine function
			// 2.) Add signatures using secp256k1_scalar_add for each of the two 32
			//   byte components of the signature
			// 3.) Verify: secp256k1_schnorrsig_verify(
			//                ctx.secp(),
			//                &agg_sig as *const u8,
			//                &msg as *const u8,
			//                msg_len,
			//                &agg_pk as *const u8,
			//        )

			let mut pubkey1 = [0u8; 64];
			let mut pubkey2 = [0u8; 64];
			let mut aggpk = [0u8; 64];
			let mut aggpk_xonly = [0u8; 32];
			let mut pk_parity = -1i32;

			assert_eq!(
				secp256k1_keypair_pub(ctx.secp(), &mut pubkey1 as *mut u8, &keypair1 as *const u8),
				1
			);

			assert_eq!(
				secp256k1_keypair_pub(ctx.secp(), &mut pubkey2 as *mut u8, &keypair2 as *const u8),
				1
			);

			let ins: &[*const u8] = &[pubkey1.as_ptr(), pubkey2.as_ptr()];

			assert_eq!(
				secp256k1_ec_pubkey_combine(ctx.secp(), &mut aggpk as *mut u8, ins.as_ptr(), 2),
				1
			);

			assert_eq!(
				secp256k1_xonly_pubkey_from_pubkey(
					ctx.secp(),
					&mut aggpk_xonly as *mut u8,
					&mut pk_parity as *mut i32,
					&aggpk as *const u8
				),
				1
			);

			//for i in 0..64 {
			let i = 0;
			println!("sig1[{}]={},sig2[{}]={}", i, sig1[i], i, sig2[i]);
			let i = 31;
			println!("sig1[{}]={},sig2[{}]={}", i, sig1[i], i, sig2[i]);
			let i = 32;
			println!("sig1[{}]={},sig2[{}]={}", i, sig1[i], i, sig2[i]);
			let i = 63;
			println!("sig1[{}]={},sig2[{}]={}", i, sig1[i], i, sig2[i]);
			//}
			let mut agg_sig = [0u8; 64];
		}
	}
}
