use core::convert::{AsMut, AsRef};
use core::ptr::{copy_nonoverlapping, null, null_mut};
use crypto::sha3::SHA3_512;
use crypto::{
	cpsrng_context_create, cpsrng_rand_bytes_ctx, safe_secp256k1_context_create,
	safe_secp256k1_context_destroy, secp256k1_ec_seckey_negate, secp256k1_keypair_create,
	secp256k1_keypair_xonly_pub, secp256k1_musig_nonce_agg, secp256k1_musig_nonce_gen,
	secp256k1_musig_nonce_process, secp256k1_musig_partial_sig_verify,
	secp256k1_musig_partial_sign, secp256k1_musig_pubkey_agg, secp256k1_pedersen_commit,
	secp256k1_xonly_pubkey_parse, secp256k1_xonly_pubkey_serialize, GENERATOR_H,
	SECP256K1_CONTEXT_NONE,
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

	pub fn to_keypair(&self, secp: &mut Secp) -> [u8; 96] {
		let mut keypair = [0u8; 96];
		unsafe {
			secp256k1_keypair_create(secp.ctx, &mut keypair as *mut u8, &self.0 as *const u8);
		}
		keypair
	}
}

#[derive(Clone, Copy)]
pub struct PublicKey([u8; 32]);

impl AsRef<[u8]> for PublicKey {
	fn as_ref(&self) -> &[u8] {
		&self.0
	}
}

impl AsMut<[u8]> for PublicKey {
	fn as_mut(&mut self) -> &mut [u8] {
		&mut self.0
	}
}

impl PublicKey {
	pub fn from_private(secp: &mut Secp, key: PrivateKey) -> Self {
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

	pub fn to_pub64(&self, secp: &mut Secp) -> [u8; 64] {
		let mut ret = [0u8; 64];
		unsafe {
			secp256k1_xonly_pubkey_parse(secp.ctx, &mut ret as *mut u8, &self.0 as *const u8, 32);
		}

		ret
	}
}

#[derive(Clone, Copy)]
pub struct Commitment([u8; 33]);

impl Commitment {
	pub fn generate(secp: &mut Secp, v: u64) -> Result<(Self, PrivateKey), Error> {
		match PrivateKey::generate(secp) {
			Ok(blinding) => Ok((Self::commit(secp, v, blinding), blinding)),
			Err(e) => Err(e),
		}
	}

	pub fn commit(secp: &mut Secp, v: u64, blinding: PrivateKey) -> Self {
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

#[derive(Clone, Copy)]
pub struct Session {
	keyagg_cache: [u8; 256],
	session: [u8; 256],
	session_id: [u8; 32],
	agg_pk: [u8; 64],
}

impl Session {
	pub fn new(secp: &mut Secp) -> Self {
		let keyagg_cache = [0u8; 256];
		let session = [0u8; 256];
		let agg_pk = [0u8; 64];
		Self::from_parts(secp, keyagg_cache, session, agg_pk)
	}

	pub fn from_parts(
		secp: &mut Secp,
		keyagg_cache: [u8; 256],
		session: [u8; 256],
		agg_pk: [u8; 64],
	) -> Self {
		let mut session_id = [0u8; 32];

		unsafe {
			cpsrng_rand_bytes_ctx(secp.rand, &mut session_id as *mut u8, 32);
		}
		Self {
			keyagg_cache,
			session,
			session_id,
			agg_pk,
		}
	}

	pub fn keyagg_cache_ptr(&mut self) -> *mut u8 {
		&mut self.keyagg_cache as *mut u8
	}

	pub fn session_ptr(&mut self) -> *mut u8 {
		&mut self.session as *mut u8
	}

	pub fn session_id_ptr(&self) -> *const u8 {
		&self.session_id as *const u8
	}
}

#[derive(Clone, Copy)]
pub struct Signature([u8; 64]);

impl AsRef<[u8]> for Signature {
	fn as_ref(&self) -> &[u8] {
		&self.0
	}
}

impl From<[u8; 64]> for Signature {
	fn from(bytes: [u8; 64]) -> Self {
		Self(bytes)
	}
}

impl Signature {
	pub fn sign(secp: &mut Secp, session: &mut Session, skey: PrivateKey, _value: &[u8]) -> Self {
		let mut sig = [0u8; 64];
		let mut secnonce = [0u8; 132];
		let mut pubnonce = [0u8; 132];
		let _aggnonce = [0u8; 132];
		let mut pubkey = [0u8; 64];
		let mut pk_parity = 0;

		unsafe {
			cpsrng_rand_bytes_ctx(secp.rand, &mut secnonce as *mut u8, 132);
			let keypair = skey.to_keypair(secp);
			secp256k1_keypair_xonly_pub(
				secp.ctx,
				&mut pubkey as *mut u8,
				&mut pk_parity as *mut i32,
				&keypair as *const u8,
			);
			secp256k1_musig_nonce_gen(
				secp.ctx,
				&mut secnonce as *mut u8,
				&mut pubnonce as *mut u8,
				session.session_id_ptr(),
				skey.as_ref().as_ptr(),
				&pubkey as *const u8,
				null(),
				null(),
				null_mut(),
			);

			//secp256k1_musig_nonce_agg(secp.ctx, aggnonce,

			secp256k1_musig_partial_sign(
				secp.ctx,
				&mut sig as *mut u8,
				&secnonce as *const u8,
				&keypair as *const u8,
				session.keyagg_cache_ptr(),
				session.session_ptr(),
			);
		}
		Signature(sig)
	}

	pub fn verify(
		&self,
		secp: &mut Secp,
		session: &mut Session,
		pubnonce: [u8; 132],
		pkey: PublicKey,
	) -> bool {
		unsafe {
			secp256k1_musig_partial_sig_verify(
				secp.ctx,
				&self.0 as *const u8,
				&pubnonce as *const u8,
				pkey.to_pub64(secp).as_ptr(),
				session.keyagg_cache_ptr(),
				session.session_ptr(),
			) == 1
		}
	}
}

pub struct Kernel {
	offset: [u8; 32],
	excess: [u8; 32],
	signature: [u8; 64],
}

pub struct SecNonce([u8; 132]);

impl AsRef<[u8]> for SecNonce {
	fn as_ref(&self) -> &[u8] {
		&self.0
	}
}

impl SecNonce {
	pub fn from_seed(seed: &[u8]) -> Self {
		if seed.len() < 32 {
			exit!("seed len must be >= 32");
		}
		let mut v = [0u8; 132];
		unsafe {
			copy_nonoverlapping(seed.as_ptr(), v.as_mut_ptr(), 32);
		}
		let mut sha3_512 = SHA3_512::new();
		sha3_512.update(seed);
		let v1 = sha3_512.finalize();
		unsafe {
			copy_nonoverlapping(v1.as_ptr(), v.as_mut_ptr().add(32), 64);
		}
		let mut sha3_512 = SHA3_512::new();
		sha3_512.update(v1.as_ref());
		let v2 = sha3_512.finalize();
		unsafe {
			copy_nonoverlapping(v2.as_ptr(), v.as_mut_ptr().add(96), 36);
		}
		Self(v)
	}
}

pub struct Transaction {
	outputs: Vec<Commitment>,
	pubnonces: Vec<[u8; 132]>,
	pubkeys: Vec<PublicKey>,
	inputs: Vec<Commitment>,
	kernel: Kernel,
	keyagg_cache: [u8; 256],
	session: [u8; 256],
	partial_signatures: Vec<[u8; 32]>,
	amount: u64,
}

impl Transaction {
	pub fn new() -> Self {
		Self {
			outputs: Vec::new(),
			pubnonces: Vec::new(),
			pubkeys: Vec::new(),
			inputs: Vec::new(),
			kernel: Kernel {
				offset: [0u8; 32],
				excess: [0u8; 32],
				signature: [0u8; 64],
			},
			keyagg_cache: [0u8; 256],
			session: [0u8; 256],
			partial_signatures: Vec::new(),
			amount: 0,
		}
	}

	pub fn invoice(
		&mut self,
		secp: &mut Secp,
		amount: u64,
		blinding: PrivateKey,
		session_id: [u8; 32],
	) -> Result<(), Error> {
		let output = Commitment::commit(secp, amount, blinding);
		match self.outputs.push(output) {
			Ok(_) => {}
			Err(e) => return Err(e),
		}
		let mut pubnonce = [0u8; 132];
		let secnonce = SecNonce::from_seed(blinding.as_ref());
		let pubkey = PublicKey::from_private(secp, blinding);

		unsafe {
			if secp256k1_musig_nonce_gen(
				secp.ctx,
				secnonce.as_ref().as_ptr(),
				pubnonce.as_mut_ptr(),
				session_id.as_ptr(),
				blinding.as_ref().as_ptr(),
				pubkey.to_pub64(secp).as_ptr(),
				null(),
				null(),
				null_mut(),
			) != 0
			{
				return Err(err!(SecpErr));
			}
			// invert pubkey
			use core::clone::Clone;
			let mut inverted_blinding = blinding.clone();
			secp256k1_ec_seckey_negate(secp.ctx, inverted_blinding.0.as_mut_ptr());
			let inverted_pub = PublicKey::from_private(secp, inverted_blinding);
			match self.pubkeys.push(inverted_pub) {
				Ok(_) => {}
				Err(e) => return Err(e),
			}
		}
		match self.pubnonces.push(pubnonce) {
			Ok(_) => {
				self.amount = amount;
				Ok(())
			}
			Err(e) => Err(e),
		}
	}

	pub fn pay(
		&mut self,
		secp: &mut Secp,
		inputs: &[(PrivateKey, u64)],
		change_key: Option<PrivateKey>,
		session_id: [u8; 32],
	) -> Result<(), Error> {
		let mut input_sum = 0;
		for i in 0..inputs.len() {
			input_sum += inputs[i].1;
		}
		if input_sum < self.amount {
			return Err(err!(InsufficientFunds));
		} else if input_sum > self.amount {
			let change_key = match change_key {
				Some(change_key) => change_key,
				None => return Err(err!(IllegalArgument)),
			};
			let diff_amt = input_sum - self.amount;
			let output = Commitment::commit(secp, diff_amt, change_key);

			let mut pubnonce = [0u8; 132];
			let secnonce = SecNonce::from_seed(change_key.as_ref());
			let pubkey = PublicKey::from_private(secp, change_key);

			unsafe {
				if secp256k1_musig_nonce_gen(
					secp.ctx,
					secnonce.as_ref().as_ptr(),
					pubnonce.as_mut_ptr(),
					session_id.as_ptr(),
					change_key.as_ref().as_ptr(),
					pubkey.to_pub64(secp).as_ptr(),
					null(),
					null(),
					null_mut(),
				) != 0
				{
					return Err(err!(SecpErr));
				}

				// invert pubkey
				use core::clone::Clone;
				let mut inverted_blinding = change_key.clone();
				secp256k1_ec_seckey_negate(secp.ctx, inverted_blinding.0.as_mut_ptr());
				let inverted_pub = PublicKey::from_private(secp, inverted_blinding);
				match self.pubkeys.push(inverted_pub) {
					Ok(_) => {}
					Err(e) => return Err(e),
				}

				match self.outputs.push(output) {
					Ok(_) => {}
					Err(e) => return Err(e),
				}
			}

			match self.pubnonces.push(pubnonce) {
				Ok(_) => {}
				Err(e) => return Err(e),
			}
		}

		for (skey, amount) in inputs {
			let input = Commitment::commit(secp, *amount, *skey);
			match self.inputs.push(input) {
				Ok(_) => {}
				Err(e) => return Err(e),
			}

			let mut pubnonce = [0u8; 132];
			let secnonce = SecNonce::from_seed(skey.as_ref());
			let pubkey = PublicKey::from_private(secp, *skey);

			unsafe {
				if secp256k1_musig_nonce_gen(
					secp.ctx,
					secnonce.as_ref().as_ptr(),
					pubnonce.as_mut_ptr(),
					session_id.as_ptr(),
					skey.as_ref().as_ptr(),
					pubkey.to_pub64(secp).as_ptr(),
					null(),
					null(),
					null_mut(),
				) != 0
				{
					return Err(err!(SecpErr));
				}
			}

			match self.pubkeys.push(pubkey) {
				Ok(_) => {}
				Err(e) => return Err(e),
			}

			match self.pubnonces.push(pubnonce) {
				Ok(_) => {}
				Err(e) => return Err(e),
			}
		}

		let mut aggnonce = [0u8; 132];
		let mut agg_pk = [0u8; 64]; // Aggregated public key
		let mut pubnonce_vec = Vec::new();
		for i in 0..self.pubnonces.len() {
			match pubnonce_vec.push(&self.pubnonces[i] as *const u8) {
				Ok(_) => {}
				Err(e) => return Err(e),
			}
		}
		let mut pks_vec = Vec::new();
		for i in 0..self.pubkeys.len() {
			match pks_vec.push(self.pubkeys[i].as_ref().as_ptr()) {
				Ok(_) => {}
				Err(e) => return Err(e),
			}
		}

		let pubnonce_ptrs: &[*const u8] = pubnonce_vec.as_slice();
		let pks_ptrs = &pks_vec.as_slice();

		unsafe {
			secp256k1_musig_nonce_agg(
				secp.ctx,
				&mut aggnonce as *mut u8,
				pubnonce_ptrs.as_ptr(),
				pubnonce_ptrs.len(),
			);

			secp256k1_musig_pubkey_agg(
				secp.ctx,
				null_mut(),
				&mut agg_pk as *mut u8,
				&mut self.keyagg_cache as *mut u8,
				pks_ptrs.as_ptr(),
				pks_ptrs.len(),
			);
			let msg = [8u8; 32];

			secp256k1_musig_nonce_process(
				secp.ctx,
				&mut self.session as *mut u8,
				&aggnonce as *const u8,
				&msg as *const u8,
				&mut self.keyagg_cache as *mut u8,
				null(),
			);
		}

		Ok(())
	}

	pub fn finalize(&mut self) {}
}

#[cfg(test)]
mod test {
	use super::*;
	#[test]
	fn test_pubkey_gen() {
		let mut secp = Secp::new().unwrap();
		let sk = PrivateKey::generate(&mut secp).unwrap();
		let pk = PublicKey::from_private(&mut secp, sk);
		assert_eq!(pk.as_ref().len(), 32);
		assert_eq!(pk.to_pub64(&mut secp).len(), 64);

		let mut keypair = [0u8; 96];
		let mut pubkey = [255u8; 64];
		let mut pk_parity = 5;
		unsafe {
			assert_eq!(
				secp256k1_keypair_create(secp.ctx, &mut keypair as *mut u8, sk.as_ref().as_ptr()),
				1
			);
			assert_eq!(
				secp256k1_keypair_xonly_pub(
					secp.ctx,
					&mut pubkey as *mut u8,
					&mut pk_parity,
					&keypair as *const u8
				),
				1
			);

			assert_eq!(pk_parity, 0);
			assert_eq!(pubkey, pk.to_pub64(&mut secp));
		}
	}

	#[test]
	fn test_transaction1() {
		let mut secp = Secp::new().unwrap();
		let sk = PrivateKey::generate(&mut secp).unwrap();
		let _pk = PublicKey::from_private(&mut secp, sk);
		let _session = Session::new(&mut secp);

		let _value: &[u8] = &[0, 1, 2, 3];
	}
}
