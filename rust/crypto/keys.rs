use crypto::context::Context;
use crypto::ffi::*;
use prelude::*;

#[derive(Clone, Copy)]
pub struct PrivateKey([u8; 32]);

impl AsRef<[u8]> for PrivateKey {
	fn as_ref(&self) -> &[u8] {
		&self.0
	}
}

impl PrivateKey {
	pub fn generate(context: &mut Context) -> Result<Self, Error> {
		let mut v = [0u8; 32];
		unsafe {
			cpsrng_rand_bytes_ctx(context.rand(), &mut v as *mut u8, 32);
			if secp256k1_ec_seckey_verify(context.secp(), &v as *const u8) == 0 {
				return Err(err!(SecpErr));
			}
		}
		Ok(PrivateKey(v))
	}

	pub fn from_bytes(context: &mut Context, v: [u8; 32]) -> Result<Self, Error> {
		unsafe {
			if secp256k1_ec_seckey_verify(context.secp(), &v as *const u8) == 0 {
				return Err(err!(SecpErr));
			}
		}

		Ok(Self(v))
	}

	pub fn negate(&self, context: &mut Context) -> Result<Self, Error> {
		use core::clone::Clone;
		let mut nkey = self.0.clone();
		unsafe {
			if secp256k1_ec_seckey_negate(context.secp(), nkey.as_mut_ptr()) == 0 {
				return Err(err!(SecpErr));
			}
		}
		Ok(Self(nkey))
	}
}

#[derive(Clone, Copy)]
pub struct PublicKey([u8; 33]);

impl AsRef<[u8]> for PublicKey {
	fn as_ref(&self) -> &[u8] {
		&self.0
	}
}

impl PublicKey {
	pub fn from(ctx: &mut Context, key: PrivateKey) -> Self {
		let mut pubkey = [0u8; 64];
		let mut pubkey_out = [0u8; 33];

		let len = 33usize;

		unsafe {
			secp256k1_ec_pubkey_create(ctx.secp(), pubkey.as_mut_ptr(), key.as_ref().as_ptr());
			secp256k1_ec_pubkey_serialize(
				ctx.secp(),
				&mut pubkey_out as *mut u8,
				&len as *const usize,
				&pubkey as *const u8,
				SECP256K1_EC_COMPRESSED,
			);
		}
		PublicKey(pubkey_out)
	}

	pub fn to_pub64(&self, ctx: &mut Context) -> [u8; 64] {
		let mut ret = [0u8; 64];
		let v = unsafe {
			secp256k1_ec_pubkey_parse(ctx.secp(), &mut ret as *mut u8, &self.0 as *const u8, 33)
		};
		if v == 0 {
			// should not get here because we check these things in creation of the
			// PrivateKey.
			exit!("could not parse pubkey");
		}
		ret
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn test_skey() {
		let mut ctx = Context::new().unwrap();
		let pkey = PrivateKey::generate(&mut ctx);
		assert!(pkey.is_ok());
		let pkey = pkey.unwrap();

		let b = [1u8; 32]; // odd parity
		assert!(PrivateKey::from_bytes(&mut ctx, b).is_ok());

		let b = [2u8; 32]; // even parity
		assert!(PrivateKey::from_bytes(&mut ctx, b).is_ok());

		let k = [2u8; 32];
		let k1 = PrivateKey::from_bytes(&mut ctx, k).unwrap();
		assert_eq!(k1.as_ref(), k);

		let pk1 = PublicKey::from(&mut ctx, k1);
		let pk0 = PublicKey::from(&mut ctx, pkey);
		let pkarr: &[*const u8] = &[pk0.as_ref().as_ptr(), pk1.as_ref().as_ptr()];
		let mut out = [0u8; 64];
		unsafe {
			assert_eq!(
				secp256k1_ec_pubkey_combine(
					ctx.secp(),
					&mut out as *mut u8,
					pkarr.as_ptr(),
					pkarr.len(),
				),
				1,
			);
		}
	}
}
