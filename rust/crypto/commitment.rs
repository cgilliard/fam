use crypto::context::Context;
use crypto::keys::PrivateKey;
use prelude::*;

use crypto::ffi::{secp256k1_pedersen_commit, GENERATOR_H};

#[derive(Clone, Copy)]
pub struct Commitment([u8; 33]);

impl AsRef<[u8]> for Commitment {
	fn as_ref(&self) -> &[u8] {
		&self.0
	}
}

impl Commitment {
	pub fn generate(ctx: &mut Context, v: u64) -> Result<(Self, PrivateKey), Error> {
		match PrivateKey::generate(ctx) {
			Ok(blinding) => Ok((Self::commit(ctx, v, blinding), blinding)),
			Err(e) => Err(e),
		}
	}

	pub fn commit(ctx: &mut Context, v: u64, blinding: PrivateKey) -> Self {
		let mut commit = [0u8; 33];
		let x = blinding.as_ref();
		unsafe {
			secp256k1_pedersen_commit(
				ctx.secp(),
				&mut commit as *mut u8,
				x.as_ptr(),
				v,
				&GENERATOR_H as *const u8,
			);
		}
		Commitment(commit)
	}

	pub fn from(bytes: [u8; 33]) -> Self {
		Self(bytes)
	}
}
