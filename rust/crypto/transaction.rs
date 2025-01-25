use core::convert::AsRef;
use crypto::{
	cpsrng_context_create, cpsrng_rand_bytes_ctx, safe_secp256k1_context_create,
	safe_secp256k1_context_destroy, secp256k1_ec_seckey_negate, secp256k1_keypair_create,
	secp256k1_keypair_xonly_pub, secp256k1_pedersen_commit, secp256k1_xonly_pubkey_serialize,
	GENERATOR_H, SECP256K1_CONTEXT_NONE,
};
use prelude::*;

pub struct Secp {
	ctx: *mut u8,
	rand: *mut u8,
}

impl Drop for Secp {
	fn drop(&mut self) {
		safe_secp256k1_context_destroy(self.ctx);
	}
}

impl Secp {
	pub fn new() -> Result<Self, Error> {
		let ctx = safe_secp256k1_context_create(SECP256K1_CONTEXT_NONE);
		if ctx.is_null() {
			Err(err!(SecpInit))
		} else {
			let rand = unsafe { cpsrng_context_create() };
			if rand.is_null() {
				safe_secp256k1_context_destroy(ctx);
			}
			Ok(Self { ctx, rand })
		}
	}
}

#[derive(Clone, Copy)]
pub struct PrivateKey([u8; 32]);

impl AsRef<[u8]> for PrivateKey {
	fn as_ref(&self) -> &[u8] {
		&self.0
	}
}

impl PrivateKey {
	pub fn generate(secp: &mut Secp) -> Result<Self, Error> {
		let mut v = [0u8; 32];
		let mut keypair = [0u8; 96];
		let mut pk_parity = 0;
		let mut pubkey = [0u8; 64];

		unsafe {
			cpsrng_rand_bytes_ctx(secp.rand, &mut v as *mut u8, 32);
			if secp256k1_keypair_create(secp.ctx, &mut keypair as *mut u8, &v as *const u8) == 0 {
				return Err(err!(SecpErr));
			}
			if secp256k1_keypair_xonly_pub(
				secp.ctx,
				&mut pubkey as *mut u8,
				&mut pk_parity,
				&keypair as *const u8,
			) == 0
			{
				return Err(err!(SecpErr));
			}

			if pk_parity == 1 {
				if secp256k1_ec_seckey_negate(secp.ctx, &mut v as *mut u8) == 0 {
					return Err(err!(SecpErr));
				}
			}
		}
		Ok(PrivateKey(v))
	}

	pub fn from(bytes: [u8; 32]) -> Self {
		PrivateKey(bytes)
	}
}

#[derive(Clone, Copy)]
pub struct PublicKey([u8; 32]);

impl AsRef<[u8]> for PublicKey {
	fn as_ref(&self) -> &[u8] {
		&self.0
	}
}

impl PublicKey {
	pub fn from_private(key: PrivateKey, secp: Secp) -> Self {
		let mut pubkey = [0u8; 64];
		let mut pubkey_out = [0u8; 32];
		let mut keypair = [0u8; 96];
		let mut pk_parity = 0i32;
		unsafe {
			secp256k1_keypair_create(secp.ctx, &mut keypair as *mut u8, key.as_ref().as_ptr());
			secp256k1_keypair_xonly_pub(
				secp.ctx,
				&mut pubkey as *mut u8,
				&mut pk_parity,
				&keypair as *const u8,
			);
			secp256k1_xonly_pubkey_serialize(
				secp.ctx,
				&mut pubkey_out as *mut u8,
				&pubkey as *const u8,
			);
		}
		PublicKey(pubkey_out)
	}
}

#[derive(Clone, Copy)]
pub struct Commitment([u8; 33]);

impl Commitment {
	pub fn generate(v: u64, secp: &mut Secp) -> Result<(Self, PrivateKey), Error> {
		match PrivateKey::generate(secp) {
			Ok(blinding) => Ok((Self::commit(v, blinding, secp), blinding)),
			Err(e) => Err(e),
		}
	}

	pub fn commit(v: u64, blinding: PrivateKey, secp: &mut Secp) -> Self {
		let mut commit = [0u8; 33];
		let x = blinding.as_ref();
		unsafe {
			secp256k1_pedersen_commit(
				secp.ctx,
				&mut commit as *mut u8,
				x.as_ptr(),
				v,
				&GENERATOR_H as *const u8,
			);
		}
		Commitment(commit)
	}
}

pub struct TxKernel {}

pub struct Transaction {
	output_recv: Option<Commitment>,
	output_change: Option<Commitment>,
}

impl Transaction {
	pub fn new() -> Self {
		Self {
			output_recv: None,
			output_change: None,
		}
	}

	pub fn invoice(&mut self, _amount: u64, _blinding: PrivateKey) -> Result<(), Error> {
		Ok(())
	}

	pub fn pay(
		&mut self,
		_inputs: &[(PrivateKey, Commitment)],
		_change: Commitment,
	) -> Result<(), Error> {
		Ok(())
	}
}
