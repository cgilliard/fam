use core::ptr::{copy_nonoverlapping, null, null_mut};
use crypto::commitment::Commitment;
use crypto::context::Context;
use crypto::ffi::{
	cpsrng_rand_bytes_ctx, secp256k1_ec_pubkey_combine, secp256k1_keypair_create,
	secp256k1_musig_nonce_agg, secp256k1_musig_nonce_gen, secp256k1_musig_nonce_process,
	secp256k1_musig_partial_sig_agg, secp256k1_musig_partial_sign, secp256k1_musig_pubkey_agg,
};
use crypto::keys::{PrivateKey, PublicKey};
use crypto::session::Session;
use crypto::sha3::SHA3_256;
use prelude::*;

pub struct ParticipantData {
	pub public_blind_excess: [u8; 64],
	pub public_nonces: Vec<[u8; 132]>,
	pub part_sigs: Vec<[u8; 64]>,
	// Note: we do not serialize the session_ids or sec_nonces when sending to another participant
	pub sec_nonces: Vec<[u8; 132]>,
	pub session_ids: Vec<[u8; 32]>,
}

pub struct Slate {
	session: Session,
	participant_data: Vec<ParticipantData>,
	inputs: Vec<Commitment>,
	outputs: Vec<Commitment>,
	fee: u64,
}

impl Slate {
	pub fn new(ctx: &mut Context, fee: u64) -> Self {
		Self {
			session: Session::new(ctx),
			participant_data: Vec::new(),
			inputs: Vec::new(),
			outputs: Vec::new(),
			fee,
		}
	}

	pub fn add_commitments(
		&mut self,
		ctx: &mut Context,
		inputs: Vec<(u64, PrivateKey)>,
		outputs: Vec<(u64, PrivateKey)>,
	) -> Result<(), Error> {
		let mut session_ids = Vec::new();
		let mut public_nonces = Vec::new();
		let mut sec_nonces = Vec::new();
		let mut public_blind_excess = if inputs.len() > 0 {
			PublicKey::from(ctx, inputs[0].1).to_pub64(ctx)
		} else if outputs.len() > 0 {
			PublicKey::from(ctx, outputs[0].1).to_pub64(ctx)
		} else {
			return Err(err!(IllegalArgument));
		};

		let mut i = 0;
		for input in &inputs {
			let mut session_id = [0u8; 32];
			unsafe {
				cpsrng_rand_bytes_ctx(ctx.rand(), &mut session_id as *mut u8, 32);
			}

			match session_ids.push(session_id) {
				Ok(_) => {}
				Err(e) => return Err(e),
			}

			let mut v = [0u8; 132];
			unsafe {
				cpsrng_rand_bytes_ctx(ctx.rand(), &mut v as *mut u8, 132);
			}
			match sec_nonces.push(v) {
				Ok(_) => {}
				Err(e) => return Err(e),
			}

			let mut p = [0u8; 132];
			let sk = input.1.as_ref();
			let pk = PublicKey::from(ctx, input.1).to_pub64(ctx);
			unsafe {
				if secp256k1_musig_nonce_gen(
					ctx.secp(),
					&v as *const u8,
					&mut p as *mut u8,
					&session_id as *const u8,
					sk.as_ptr(),
					&pk as *const u8,
					null(),
					null(),
					null_mut(),
				) == 0
				{
					return Err(err!(SecpErr));
				}
			}
			match public_nonces.push(p) {
				Ok(_) => {}
				Err(e) => return Err(e),
			}

			let pkarr: &[*const u8] = &[pk.as_ptr(), public_blind_excess.as_ptr()];

			let mut tmp = [0u8; 64];
			unsafe {
				if i != 0 {
					if secp256k1_ec_pubkey_combine(
						ctx.secp(),
						&mut tmp as *mut u8,
						pkarr.as_ptr(),
						pkarr.len(),
					) == 0
					{
						return Err(err!(SecpErr));
					}
					copy_nonoverlapping(tmp.as_ptr(), public_blind_excess.as_mut_ptr(), 64);
				}
			}

			// check if its value is 0. If so it's just the offset, don't add the
			// input to the transaction.
			if input.0 != 0 {
				match self.inputs.push(Commitment::commit(ctx, input.0, input.1)) {
					Ok(_) => {}
					Err(e) => return Err(e),
				}
			}
			i += 1;
		}
		let mut count = 0;
		for output in &outputs {
			let mut session_id = [0u8; 32];
			unsafe {
				cpsrng_rand_bytes_ctx(ctx.rand(), &mut session_id as *mut u8, 32);
			}

			match session_ids.push(session_id) {
				Ok(_) => {}
				Err(e) => return Err(e),
			}
			let mut v = [0u8; 132];
			unsafe {
				cpsrng_rand_bytes_ctx(ctx.rand(), &mut v as *mut u8, 132);
			}
			match sec_nonces.push(v) {
				Ok(_) => {}
				Err(e) => return Err(e),
			}

			let negative_output = match output.1.negate(ctx) {
				Ok(n) => n,
				Err(e) => return Err(e),
			};

			let mut p = [0u8; 132];
			let sk = negative_output.as_ref();
			let pk = PublicKey::from(ctx, negative_output).to_pub64(ctx);
			unsafe {
				if secp256k1_musig_nonce_gen(
					ctx.secp(),
					&v as *const u8,
					&mut p as *mut u8,
					&session_id as *const u8,
					sk.as_ptr(),
					&pk as *const u8,
					null(),
					null(),
					null_mut(),
				) == 0
				{
					return Err(err!(SecpErr));
				}
			}
			match public_nonces.push(p) {
				Ok(_) => {}
				Err(e) => return Err(e),
			}

			if count > 0 || inputs.len() == 0 {
				let pk = PublicKey::from(ctx, negative_output).to_pub64(ctx);
				let pkarr: &[*const u8] = &[pk.as_ptr(), public_blind_excess.as_ptr()];
				unsafe {
					if secp256k1_ec_pubkey_combine(
						ctx.secp(),
						&mut public_blind_excess as *mut u8,
						pkarr.as_ptr(),
						2,
					) == 0
					{
						return Err(err!(SecpErr));
					}
				}
			}

			match self
				.outputs
				.push(Commitment::commit(ctx, output.0, output.1))
			{
				Ok(_) => {}
				Err(e) => return Err(e),
			}

			count += 1;
		}
		let pd = ParticipantData {
			public_nonces,
			public_blind_excess,
			sec_nonces,
			session_ids,
			part_sigs: Vec::new(),
		};
		match self.participant_data.push(pd) {
			Ok(_) => {}
			Err(e) => return Err(e),
		};

		Ok(())
	}

	pub fn sign_index(
		&mut self,
		index: usize,
		ctx: &mut Context,
		inputs: Vec<PrivateKey>,
		outputs: Vec<PrivateKey>,
	) -> Result<(), Error> {
		if self.participant_data.len() == 0 || index >= self.participant_data.len() {
			return Err(err!(IllegalState));
		}

		let aggnonce = match self.aggnonce(ctx) {
			Ok(v) => v,
			Err(e) => return Err(e),
		};
		let public_blind_excess_sum = match self.public_blind_excess_sum(ctx) {
			Ok(v) => v,
			Err(e) => return Err(e),
		};
		let pd = &mut self.participant_data[index];

		let mut sha3_256 = SHA3_256::new();
		let mut feebytes = [0u8; 8];
		to_le_bytes_u64(self.fee, &mut feebytes);
		sha3_256.update(feebytes);
		sha3_256.update(public_blind_excess_sum.as_ref());
		let msg: [u8; 32] = sha3_256.finalize();
		unsafe {
			println!("1");
			if secp256k1_musig_nonce_process(
				ctx.secp(),
				self.session.session_ptr(),
				aggnonce.as_ptr(),
				msg.as_ptr(),
				self.session.keyagg_cache_ptr(),
				null(),
			) == 0
			{
				return Err(err!(SecpErr));
			}

			println!("x");

			let mut i = 0;
			for input in inputs {
				let secnonce = pd.sec_nonces[i];
				let mut partial_sig = [0u8; 64];
				let mut keypair = [0u8; 96];
				secp256k1_keypair_create(
					ctx.secp(),
					&mut keypair as *mut u8,
					input.as_ref().as_ptr(),
				);
				println!("pre");

				secp256k1_musig_partial_sign(
					ctx.secp(),
					&mut partial_sig as *mut u8,
					&secnonce as *const u8,
					&keypair as *const u8,
					self.session.keyagg_cache_ptr(),
					self.session.session_ptr(),
				);
				i += 1;
			}
			println!("2");

			for output in outputs {
				let negative_output = match output.negate(ctx) {
					Ok(n) => n,
					Err(e) => return Err(e),
				};

				let secnonce = pd.sec_nonces[i];
				let mut partial_sig = [0u8; 64];
				let mut keypair = [0u8; 96];
				secp256k1_keypair_create(
					ctx.secp(),
					&mut keypair as *mut u8,
					negative_output.as_ref().as_ptr(),
				);

				secp256k1_musig_partial_sign(
					ctx.secp(),
					&mut partial_sig as *mut u8,
					&secnonce as *const u8,
					&keypair as *const u8,
					self.session.keyagg_cache_ptr(),
					self.session.session_ptr(),
				);
				match pd.part_sigs.push(partial_sig) {
					Ok(_) => {}
					Err(e) => return Err(e),
				}
				i += 1;
			}
		}

		Ok(())
	}

	pub fn finalize(&mut self, ctx: &mut Context) -> Result<(), Error> {
		let mut sig_vec = Vec::new();
		for pd in &self.participant_data {
			for sig in &pd.part_sigs {
				match sig_vec.push(sig.as_ptr()) {
					Ok(_) => {}
					Err(e) => return Err(e),
				}
			}
		}

		let sig_slice = sig_vec.as_slice();
		let mut final_sig = [0u8; 64];

		unsafe {
			secp256k1_musig_partial_sig_agg(
				ctx.secp(),
				&mut final_sig as *mut u8,
				self.session.session_ptr(),
				sig_slice.as_ptr(),
				sig_slice.len(),
			);
		}

		Ok(())
	}

	fn aggnonce(&self, ctx: &mut Context) -> Result<[u8; 132], Error> {
		let mut aggnonce = [0u8; 132];
		let mut count = 0;
		let mut pub_nonces = Vec::new();
		for pd in &self.participant_data {
			for pn in &pd.public_nonces {
				match pub_nonces.push(pn) {
					Ok(_) => {}
					Err(e) => return Err(e),
				}
				count += 1;
			}
		}

		let mut pub_nonce_ptr_vec = Vec::new();
		println!("pub_nonces.len={}", pub_nonces.len());
		for nonce in &pub_nonces {
			match pub_nonce_ptr_vec.push(nonce.as_ptr()) {
				Ok(_) => {}
				Err(e) => return Err(e),
			}
		}
		println!("complete");
		let pub_nonces_ptr: &[*const u8] = pub_nonce_ptr_vec.as_slice();

		unsafe {
			if secp256k1_musig_nonce_agg(
				ctx.secp(),
				&mut aggnonce as *mut u8,
				pub_nonces_ptr.as_ptr(),
				count,
			) == 0
			{
				return Err(err!(SecpErr));
			}
		}
		println!("aggnonce");

		Ok(aggnonce)
	}

	fn public_blind_excess_sum(&mut self, ctx: &mut Context) -> Result<[u8; 64], Error> {
		let mut v = [0u8; 64];
		let mut pbe_vec = Vec::new();
		for pd in &self.participant_data {
			match pbe_vec.push(pd.public_blind_excess.as_ptr()) {
				Ok(_) => {}
				Err(e) => return Err(e),
			}
		}
		let pbe: &[*const u8] = pbe_vec.as_slice();

		unsafe {
			if secp256k1_musig_pubkey_agg(
				ctx.secp(),
				null_mut(),
				&mut v as *mut u8,
				self.session.keyagg_cache_ptr(),
				pbe.as_ptr(),
				self.participant_data.len(),
			) == 0
			{
				return Err(err!(SecpErr));
			}
		}

		Ok(v)
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn test_tx1() {
		let mut ctx = Context::new().unwrap();
		let mut slate = Slate::new(&mut ctx, 100);
		let pkinput = PrivateKey::generate(&mut ctx).unwrap();
		let _pkoutput = PrivateKey::generate(&mut ctx).unwrap();
		let _offset = PrivateKey::generate(&mut ctx).unwrap();

		slate
			.add_commitments(&mut ctx, vec![(300, pkinput)].unwrap(), Vec::new())
			.unwrap();

		/*
		slate
			.add_commitments(
				&mut ctx,
				vec![(0, offset)].unwrap(),
				vec![(300, pkoutput)].unwrap(),
			)
			.unwrap();
			*/

		//	slate.sign_index(0, &mut ctx, vec![pkinput].unwrap(), Vec::new());

		//slate.finalize(&mut ctx);
	}
}
