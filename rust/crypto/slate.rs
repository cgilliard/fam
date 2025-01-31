use core::ptr::{copy_nonoverlapping, null, null_mut};
use crypto::commitment::Commitment;
use crypto::context::Context;
use crypto::ffi::{
	cpsrng_rand_bytes_ctx, secp256k1_ec_pubkey_combine, secp256k1_keypair_create,
	secp256k1_musig_nonce_agg, secp256k1_musig_nonce_gen, secp256k1_musig_nonce_process,
	secp256k1_musig_partial_sig_agg, secp256k1_musig_partial_sig_verify,
	secp256k1_musig_partial_sign, secp256k1_musig_pubkey_agg, secp256k1_schnorrsig_verify,
};
use crypto::keys::{PrivateKey, PublicKey};
use crypto::session::Session;
use crypto::sha3::SHA3_256;
use prelude::*;

pub struct ParticipantData {
	pub public_blind_sum: [u8; 64],
	pub public_nonces: Vec<[u8; 132]>,
	pub part_sigs: Vec<[u8; 64]>,
}

pub struct Slate {
	session: Session,
	participant_data: Vec<ParticipantData>,
	inputs: Vec<Commitment>,
	outputs: Vec<Commitment>,
	fee: u64,
	final_sig: Option<[u8; 64]>,
	// Note: we do not serialize the session_ids or sec_nonces when sending to another participant
	sec_nonces: Vec<[u8; 132]>,
}

impl Slate {
	pub fn new(fee: u64) -> Self {
		Self {
			session: Session::new(),
			participant_data: Vec::new(),
			inputs: Vec::new(),
			outputs: Vec::new(),
			fee,
			final_sig: None,
			sec_nonces: Vec::new(),
		}
	}

	/*
	 * Format:
	 * [keyagg_cache - 256 bytes]
	 * [session - 256 bytes]
	 * [fee - 8 bytes]
	 * [input count - 4 bytes]
	 * [input1 - 33 bytes]
	 * [input2 - 33 bytes]
	 * ...
	 * [output count - 4 bytes]
	 * [output1 - 33 bytes]
	 * [output2 - 33 bytes]
	 * ...
	 * [participant_count - 4 bytes]
	 * [participant1]
	 * [participant2]
	 * ...
	 *
	 * participant format:
	 * [pub_blind_sum - 64 bytes]
	 * [public_nonce count - 4 bytes]
	 * [public nonce 1 - 132 bytes]
	 * [public nonce 2 - 132 bytes]
	 * ...
	 * [partial_sig count - 4 bytes]
	 * [partial_sig 1 - 64 bytes]
	 * [partial_sig 2 - 64 bytes]
	 * ...
	 */
	pub fn serialize(&self) -> Result<Vec<u8>, Error> {
		if self.inputs.len() >= 0xFFFFFFFF
			|| self.outputs.len() >= 0xFFFFFFFF
			|| self.participant_data.len() >= 0xFFFFFFFF
		{
			return Err(err!(Overflow));
		}

		let mut ret = Vec::new();
		match ret.append_ptr(&self.session.keyagg_cache as *const u8, 256) {
			Ok(_) => {}
			Err(e) => return Err(e),
		}

		match ret.append_ptr(&self.session.session as *const u8, 256) {
			Ok(_) => {}
			Err(e) => return Err(e),
		}

		let mut feebytes = [0u8; 8];
		to_le_bytes_u64(self.fee, &mut feebytes);
		match ret.append_ptr(&feebytes as *const u8, 8) {
			Ok(_) => {}
			Err(e) => return Err(e),
		}
		let mut input_count_bytes = [0u8; 4];
		to_le_bytes_u32(self.inputs.len() as u32, &mut input_count_bytes);

		match ret.append_ptr(&input_count_bytes as *const u8, 4) {
			Ok(_) => {}
			Err(e) => return Err(e),
		}

		for input in &self.inputs {
			match ret.append_ptr(input.as_ref().as_ptr(), 33) {
				Ok(_) => {}
				Err(e) => return Err(e),
			}
		}

		let mut output_count_bytes = [0u8; 4];
		to_le_bytes_u32(self.outputs.len() as u32, &mut output_count_bytes);

		match ret.append_ptr(&output_count_bytes as *const u8, 4) {
			Ok(_) => {}
			Err(e) => return Err(e),
		}

		for output in &self.outputs {
			match ret.append_ptr(output.as_ref().as_ptr(), 33) {
				Ok(_) => {}
				Err(e) => return Err(e),
			}
		}

		let mut participant_count_bytes = [0u8; 4];
		to_le_bytes_u32(
			self.participant_data.len() as u32,
			&mut participant_count_bytes,
		);

		match ret.append_ptr(&participant_count_bytes as *const u8, 4) {
			Ok(_) => {}
			Err(e) => return Err(e),
		}

		for participant in &self.participant_data {
			match ret.append_ptr(&participant.public_blind_sum as *const u8, 64) {
				Ok(_) => {}
				Err(e) => return Err(e),
			}

			let mut public_nonces_count = [0u8; 4];
			to_le_bytes_u32(
				participant.public_nonces.len() as u32,
				&mut public_nonces_count,
			);

			match ret.append_ptr(&public_nonces_count as *const u8, 4) {
				Ok(_) => {}
				Err(e) => return Err(e),
			}

			for nonce in &participant.public_nonces {
				match ret.append_ptr(nonce.as_ref().as_ptr(), 132) {
					Ok(_) => {}
					Err(e) => return Err(e),
				}
			}

			let mut part_sig_count = [0u8; 4];
			to_le_bytes_u32(participant.part_sigs.len() as u32, &mut part_sig_count);

			match ret.append_ptr(&part_sig_count as *const u8, 4) {
				Ok(_) => {}
				Err(e) => return Err(e),
			}

			for sig in &participant.part_sigs {
				match ret.append_ptr(sig.as_ref().as_ptr(), 64) {
					Ok(_) => {}
					Err(e) => return Err(e),
				}
			}
		}

		Ok(ret)
	}

	/*
	 * Format:
	 * [keyagg_cache - 256 bytes]
	 * [session - 256 bytes]
	 * [fee - 8 bytes]
	 * [input count - 4 bytes]
	 * [input1 - 33 bytes]
	 * [input2 - 33 bytes]
	 * ...
	 * [output count - 4 bytes]
	 * [output1 - 33 bytes]
	 * [output2 - 33 bytes]
	 * ...
	 * [participant_count - 4 bytes]
	 * [participant1]
	 * [participant2]
	 * ...
	 *
	 * participant format:
	 * [pub_blind_sum - 64 bytes]
	 * [public_nonce count - 4 bytes]
	 * [public nonce 1 - 132 bytes]
	 * [public nonce 2 - 132 bytes]
	 * ...
	 * [partial_sig count - 4 bytes]
	 * [partial_sig 1 - 64 bytes]
	 * [partial_sig 2 - 64 bytes]
	 * ...
	 */
	pub fn deserialize(bytes: Vec<u8>) -> Result<Self, Error> {
		let mut session = [0u8; 256];
		let mut keyagg_cache = [0u8; 256];
		let mut feebytes = [0u8; 8];

		let bytes_len = bytes.len();
		let bytes_ptr = bytes.as_ptr();

		if bytes_len <= 520 {
			return Err(err!(CorruptedData));
		}
		unsafe {
			copy_nonoverlapping(bytes_ptr, keyagg_cache.as_mut_ptr(), 256);
			copy_nonoverlapping(bytes_ptr.add(256), session.as_mut_ptr(), 256);
			copy_nonoverlapping(bytes_ptr.add(512), feebytes.as_mut_ptr(), 8);
		}
		let session = Session::from_parts(keyagg_cache, session);
		let fee = from_le_bytes_u64(&feebytes);
		let mut ret = Self::new(fee);
		ret.session = session;

		let mut itt = 520;
		if bytes_len < itt + 4 {
			return Err(err!(CorruptedData));
		}
		let input_count = from_le_bytes_u32(&bytes[itt..itt + 4]);
		itt += 4;

		if itt + input_count as usize * 33 >= bytes_len {
			return Err(err!(CorruptedData));
		}
		for _i in 0..input_count {
			let mut input_bytes = [0u8; 33];
			unsafe {
				copy_nonoverlapping(bytes_ptr.add(itt), input_bytes.as_mut_ptr(), 33);
			}
			let input = Commitment::from(input_bytes);
			match ret.inputs.push(input) {
				Ok(_) => {}
				Err(e) => return Err(e),
			}
			itt += 33;
		}

		if bytes_len < itt + 4 {
			return Err(err!(CorruptedData));
		}

		let output_count = from_le_bytes_u32(&bytes[itt..itt + 4]);
		itt += 4;

		if itt + output_count as usize * 33 >= bytes_len {
			return Err(err!(CorruptedData));
		}

		for _i in 0..output_count {
			let mut output_bytes = [0u8; 33];
			unsafe {
				copy_nonoverlapping(bytes_ptr.add(itt), output_bytes.as_mut_ptr(), 33);
			}
			let output = Commitment::from(output_bytes);
			match ret.outputs.push(output) {
				Ok(_) => {}
				Err(e) => return Err(e),
			}
			itt += 33;
		}

		if bytes_len < itt + 4 {
			return Err(err!(CorruptedData));
		}

		let participants = from_le_bytes_u32(&bytes[itt..itt + 4]);
		itt += 4;

		for _i in 0..participants {
			let mut public_blind_sum = [0u8; 64];
			let mut public_nonces = Vec::new();
			let mut part_sigs = Vec::new();

			if bytes_len <= itt + 64 {
				return Err(err!(CorruptedData));
			}

			unsafe {
				copy_nonoverlapping(bytes_ptr.add(itt), public_blind_sum.as_mut_ptr(), 64);
			}
			itt += 64;

			if bytes_len <= itt + 4 {
				return Err(err!(CorruptedData));
			}
			let public_nonces_count = from_le_bytes_u32(&bytes[itt..itt + 4]);
			itt += 4;

			if itt + public_nonces_count as usize * 132 >= bytes_len {
				return Err(err!(CorruptedData));
			}

			for _j in 0..public_nonces_count {
				let mut public_nonce = [0u8; 132];
				unsafe {
					copy_nonoverlapping(bytes_ptr.add(itt), public_nonce.as_mut_ptr(), 132);
				}
				match public_nonces.push(public_nonce) {
					Ok(_) => {}
					Err(e) => return Err(e),
				}
				itt += 132;
			}
			if bytes_len < itt + 4 {
				return Err(err!(CorruptedData));
			}
			let part_sigs_count = from_le_bytes_u32(&bytes[itt..itt + 4]);
			itt += 4;

			if itt + part_sigs_count as usize * 64 > bytes_len {
				return Err(err!(CorruptedData));
			}

			for _j in 0..part_sigs_count {
				let mut part_sig = [0u8; 64];
				unsafe {
					copy_nonoverlapping(bytes_ptr.add(itt), part_sig.as_mut_ptr(), 64);
				}
				match part_sigs.push(part_sig) {
					Ok(_) => {}
					Err(e) => return Err(e),
				}
				itt += 64;
			}

			let participant = ParticipantData {
				public_blind_sum,
				public_nonces,
				part_sigs,
			};
			match ret.participant_data.push(participant) {
				Ok(_) => {}
				Err(e) => return Err(e),
			}
		}

		Ok(ret)
	}

	pub fn add_commitments(
		&mut self,
		ctx: &mut Context,
		inputs: Vec<(u64, PrivateKey)>,
		outputs: Vec<(u64, PrivateKey)>,
	) -> Result<(), Error> {
		let mut public_nonces: Vec<[u8; 132]> = Vec::new();
		let mut public_blind_sum: Option<[u8; 64]> = None;
		for input in inputs {
			match self.add_nonce(ctx, &mut public_nonces, input.1) {
				Ok(_) => {}
				Err(e) => return Err(e),
			}
			match self.add_to_pbs(ctx, input.1, &mut public_blind_sum) {
				Ok(_) => {}
				Err(e) => return Err(e),
			}
			if input.0 != 0 {
				match self.inputs.push(Commitment::commit(ctx, input.0, input.1)) {
					Ok(_) => {}
					Err(e) => return Err(e),
				}
			}
		}
		for output in outputs {
			let negative_output = match output.1.negate(ctx) {
				Ok(n) => n,
				Err(e) => return Err(e),
			};
			match self.add_nonce(ctx, &mut public_nonces, negative_output) {
				Ok(_) => {}
				Err(e) => return Err(e),
			}
			match self.add_to_pbs(ctx, negative_output, &mut public_blind_sum) {
				Ok(_) => {}
				Err(e) => return Err(e),
			}
			match self
				.outputs
				.push(Commitment::commit(ctx, output.0, output.1))
			{
				Ok(_) => {}
				Err(e) => return Err(e),
			}
		}
		match public_blind_sum {
			Some(public_blind_sum) => {
				let pd = ParticipantData {
					public_blind_sum,
					part_sigs: Vec::new(),
					public_nonces,
				};
				match self.participant_data.push(pd) {
					Ok(_) => {}
					Err(e) => return Err(e),
				}
			}
			None => return Err(err!(IllegalArgument)),
		}
		Ok(())
	}

	pub fn sign_index(
		&mut self,
		index: usize,
		ctx: &mut Context,
		inputs: Vec<PrivateKey>,
		outputs: Vec<PrivateKey>,
	) -> Result<(), Error> {
		let aggnonce = match self.aggnonce(ctx) {
			Ok(v) => v,
			Err(e) => return Err(e),
		};
		let public_blind_excess_sum = match self.public_blind_sum(ctx) {
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
			if secp256k1_musig_nonce_process(
				ctx.secp(),
				&mut self.session.session as *mut u8,
				aggnonce.as_ptr(),
				msg.as_ptr(),
				&mut self.session.keyagg_cache as *mut u8,
				null(),
			) == 0
			{
				return Err(err!(SecpErr));
			}

			let mut i = 0;
			for input in inputs {
				let secnonce = self.sec_nonces[i];
				let mut partial_sig = [0u8; 64];
				let mut keypair = [0u8; 96];
				if secp256k1_keypair_create(
					ctx.secp(),
					&mut keypair as *mut u8,
					input.as_ref().as_ptr(),
				) == 0
				{
					return Err(err!(SecpErr));
				}

				if secp256k1_musig_partial_sign(
					ctx.secp(),
					&mut partial_sig as *mut u8,
					&secnonce as *const u8,
					&keypair as *const u8,
					&mut self.session.keyagg_cache as *mut u8,
					&mut self.session.session as *mut u8,
				) == 0
				{
					return Err(err!(SecpErr));
				}
				match pd.part_sigs.push(partial_sig) {
					Ok(_) => {}
					Err(e) => return Err(e),
				};
				i += 1;
			}

			for output in outputs {
				let negative_output = match output.negate(ctx) {
					Ok(no) => no,
					Err(e) => return Err(e),
				};
				let secnonce = self.sec_nonces[i];
				let mut partial_sig = [0u8; 64];
				let mut keypair = [0u8; 96];
				if secp256k1_keypair_create(
					ctx.secp(),
					&mut keypair as *mut u8,
					negative_output.as_ref().as_ptr(),
				) == 0
				{
					return Err(err!(SecpErr));
				}

				if secp256k1_musig_partial_sign(
					ctx.secp(),
					&mut partial_sig as *mut u8,
					&secnonce as *const u8,
					&keypair as *const u8,
					&mut self.session.keyagg_cache as *mut u8,
					&mut self.session.session as *mut u8,
				) == 0
				{
					return Err(err!(SecpErr));
				}
				match pd.part_sigs.push(partial_sig) {
					Ok(_) => {}
					Err(e) => return Err(e),
				};

				i += 1;
			}
		}
		Ok(())
	}

	pub fn finalize(&mut self, ctx: &mut Context) -> Result<(), Error> {
		if self.inputs.len() == 0 && self.outputs.len() == 0 {
			return Err(err!(IllegalState));
		}
		let mut final_sig = [0u8; 64];
		let mut partial_sigs_vec = Vec::new();
		for participant in &self.participant_data {
			for sig in &participant.part_sigs {
				match partial_sigs_vec.push(sig.as_ptr()) {
					Ok(_) => {}
					Err(e) => return Err(e),
				}
			}
		}

		let partial_sigs = partial_sigs_vec.as_slice();
		unsafe {
			if secp256k1_musig_partial_sig_agg(
				ctx.secp(),
				&mut final_sig as *mut u8,
				&self.session.session as *const u8,
				partial_sigs.as_ptr(),
				partial_sigs.len(),
			) == 0
			{
				return Err(err!(SecpErr));
			}
		}
		self.final_sig = Some(final_sig);
		Ok(())
	}

	fn add_nonce(
		&mut self,
		ctx: &mut Context,
		public_nonces: &mut Vec<[u8; 132]>,
		sk: PrivateKey,
	) -> Result<(), Error> {
		let mut session_id = [0u8; 32];
		let mut v = [0u8; 132];
		let mut p = [0u8; 132];

		unsafe {
			cpsrng_rand_bytes_ctx(ctx.rand(), &mut v as *mut u8, 132);
			cpsrng_rand_bytes_ctx(ctx.rand(), &mut session_id as *mut u8, 32);
		}

		let pk = PublicKey::from(ctx, sk).to_pub64(ctx);
		let sk = sk.as_ref();
		unsafe {
			if secp256k1_musig_nonce_gen(
				ctx.secp(),
				&mut v as *mut u8,
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

		match self.sec_nonces.push(v) {
			Ok(_) => {}
			Err(e) => return Err(e),
		}

		match public_nonces.push(p) {
			Ok(_) => {}
			Err(e) => return Err(e),
		}

		Ok(())
	}

	fn add_to_pbs(
		&mut self,
		ctx: &mut Context,
		pkey: PrivateKey,
		public_blind_sum: &mut Option<[u8; 64]>,
	) -> Result<(), Error> {
		let pk = PublicKey::from(ctx, pkey).to_pub64(ctx);
		match public_blind_sum {
			Some(pbs) => {
				let mut tmp = [0u8; 64];
				let pkarr: &[*const u8] = &[pk.as_ptr(), pbs.as_ptr()];
				unsafe {
					if secp256k1_ec_pubkey_combine(
						ctx.secp(),
						&mut tmp as *mut u8,
						pkarr.as_ptr(),
						pkarr.len(),
					) == 0
					{
						return Err(err!(SecpErr));
					}
				}
				*public_blind_sum = Some(tmp);
			}
			None => {
				*public_blind_sum = Some(pk);
			}
		}
		Ok(())
	}

	fn aggnonce(&self, ctx: &mut Context) -> Result<[u8; 132], Error> {
		let mut aggnonce = [0u8; 132];
		let mut count = 0;
		let mut pub_nonces = Vec::new();
		for pd in &self.participant_data {
			for pn in &pd.public_nonces {
				match pub_nonces.push(pn.as_ptr()) {
					Ok(_) => {}
					Err(e) => return Err(e),
				}
				count += 1;
			}
		}
		let pub_nonces_ptr: &[*const u8] = pub_nonces.as_slice();

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
		Ok(aggnonce)
	}

	fn public_blind_sum(&mut self, ctx: &mut Context) -> Result<[u8; 64], Error> {
		let mut v = [0u8; 64];
		let mut pbe_vec = Vec::new();
		for pd in &self.participant_data {
			match pbe_vec.push(pd.public_blind_sum.as_ptr()) {
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
				&mut self.session.keyagg_cache as *mut u8,
				pbe.as_ptr(),
				pbe.len(),
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
		let mut slate = Slate::new(100);
		let pkinput = PrivateKey::generate(&mut ctx).unwrap();
		let pkoutput = PrivateKey::generate(&mut ctx).unwrap();
		let offset = PrivateKey::generate(&mut ctx).unwrap();

		slate
			.add_commitments(&mut ctx, vec![(300, pkinput)].unwrap(), Vec::new())
			.unwrap();

		assert_eq!(slate.inputs.len(), 1);
		assert_eq!(slate.outputs.len(), 0);
		assert_eq!(slate.participant_data.len(), 1);
		assert_eq!(
			slate.participant_data[0].public_blind_sum,
			PublicKey::from(&mut ctx, pkinput).to_pub64(&mut ctx)
		);

		let _ser = slate.serialize().unwrap();

		slate
			.add_commitments(
				&mut ctx,
				vec![(0, offset)].unwrap(),
				vec![(300, pkoutput)].unwrap(),
			)
			.unwrap();

		// offset has 0 value so it is not added as an output
		assert_eq!(slate.inputs.len(), 1);
		assert_eq!(slate.outputs.len(), 1);
		assert_eq!(slate.participant_data.len(), 2);
		let mut sumpk = [0u8; 64];
		let pkoutput_neg = pkoutput.negate(&mut ctx).unwrap();
		let pbe0 = PublicKey::from(&mut ctx, offset).to_pub64(&mut ctx);
		let pbe1 = PublicKey::from(&mut ctx, pkoutput_neg).to_pub64(&mut ctx);
		let pbe: &[*const u8] = &[pbe0.as_ptr(), pbe1.as_ptr()];
		unsafe {
			assert!(
				secp256k1_ec_pubkey_combine(ctx.secp(), &mut sumpk as *mut u8, pbe.as_ptr(), 2)
					== 1
			);
		}

		assert_eq!(slate.participant_data[1].public_blind_sum.as_ref(), sumpk);

		let mut pk = [0u8; 64];
		let mut pbe_vec = Vec::new();
		for pd in &slate.participant_data {
			assert!(pbe_vec.push(pd.public_blind_sum.as_ptr()).is_ok());
		}
		let pbe: &[*const u8] = pbe_vec.as_slice();

		unsafe {
			secp256k1_musig_pubkey_agg(
				ctx.secp(),
				null_mut(),
				&mut pk as *mut u8,
				&mut slate.session.keyagg_cache as *mut u8,
				pbe.as_ptr(),
				slate.participant_data.len(),
			);
		}

		assert_eq!(pk, slate.public_blind_sum(&mut ctx).unwrap());

		let aggnonce_test = slate.aggnonce(&mut ctx).unwrap();
		let mut aggnonce = [0u8; 132];
		let mut count = 0;
		let mut pub_nonces = Vec::new();
		for pd in &slate.participant_data {
			for pn in &pd.public_nonces {
				pub_nonces.push(pn.as_ptr()).unwrap();
				count += 1;
			}
		}
		let pub_nonces_ptr: &[*const u8] = pub_nonces.as_slice();

		unsafe {
			secp256k1_musig_nonce_agg(
				ctx.secp(),
				&mut aggnonce as *mut u8,
				pub_nonces_ptr.as_ptr(),
				count,
			);
		}

		assert_eq!(aggnonce, aggnonce_test);

		assert_eq!(slate.participant_data[0].part_sigs.len(), 0);

		assert!(slate
			.sign_index(0, &mut ctx, vec![pkinput].unwrap(), Vec::new())
			.is_ok());

		assert_eq!(slate.participant_data[0].part_sigs.len(), 1);

		unsafe {
			let pk = PublicKey::from(&mut ctx, pkinput).to_pub64(&mut ctx);
			assert_eq!(
				secp256k1_musig_partial_sig_verify(
					ctx.secp(),
					&slate.participant_data[0].part_sigs[0] as *const u8,
					&slate.participant_data[0].public_nonces[0] as *const u8,
					&pk as *const u8,
					&slate.session.keyagg_cache as *const u8,
					&slate.session.session as *const u8
				),
				1
			);
		}

		assert!(slate.finalize(&mut ctx).is_ok());
	}

	#[test]
	fn test_slate_ser() {
		let mut ctx = Context::new().unwrap();
		let mut slate = Slate::new(100);
		let pkinput = PrivateKey::generate(&mut ctx).unwrap();

		slate
			.add_commitments(&mut ctx, vec![(300, pkinput)].unwrap(), Vec::new())
			.unwrap();

		let slate_ser = slate.serialize().unwrap();
		let slate_deser = Slate::deserialize(slate_ser).unwrap();

		assert_eq!(
			slate.participant_data.len(),
			slate_deser.participant_data.len()
		);

		assert_eq!(slate.session.session, slate_deser.session.session);
		assert_eq!(slate.session.keyagg_cache, slate_deser.session.keyagg_cache);
		assert_eq!(slate.fee, slate_deser.fee);

		assert_eq!(
			slate.participant_data[0].public_blind_sum,
			slate_deser.participant_data[0].public_blind_sum
		);

		assert_eq!(
			slate.participant_data[0].public_nonces.len(),
			slate_deser.participant_data[0].public_nonces.len()
		);

		assert_eq!(
			slate.participant_data[0].public_nonces[0],
			slate_deser.participant_data[0].public_nonces[0]
		);

		assert_eq!(
			slate.participant_data[0].part_sigs.len(),
			slate_deser.participant_data[0].part_sigs.len()
		);
	}

	#[test]
	fn test_finalize() {
		let mut ctx = Context::new().unwrap();
		let mut slate = Slate::new(100);
		let pkinput = PrivateKey::generate(&mut ctx).unwrap();
		let pkoutput = PrivateKey::generate(&mut ctx).unwrap();
		let offset = PrivateKey::generate(&mut ctx).unwrap();

		slate
			.add_commitments(&mut ctx, vec![(300, pkinput)].unwrap(), Vec::new())
			.unwrap();

		let slate_ser = slate.serialize().unwrap();
		let mut slate_userb = Slate::deserialize(slate_ser).unwrap();

		slate_userb
			.add_commitments(
				&mut ctx,
				vec![(0, offset)].unwrap(),
				vec![(300, pkoutput)].unwrap(),
			)
			.unwrap();

		assert!(slate_userb
			.sign_index(1, &mut ctx, vec![offset].unwrap(), vec![pkoutput].unwrap())
			.is_ok());

		assert_eq!(slate_userb.participant_data[1].part_sigs.len(), 2);

		unsafe {
			let pk = PublicKey::from(&mut ctx, offset).to_pub64(&mut ctx);
			assert_eq!(
				secp256k1_musig_partial_sig_verify(
					ctx.secp(),
					&slate_userb.participant_data[1].part_sigs[0] as *const u8,
					&slate_userb.participant_data[1].public_nonces[0] as *const u8,
					&pk as *const u8,
					&slate_userb.session.keyagg_cache as *const u8,
					&slate_userb.session.session as *const u8
				),
				1
			);
			let negavite_pkoutput = pkoutput.negate(&mut ctx).unwrap();
			let pk = PublicKey::from(&mut ctx, negavite_pkoutput);
			let pk = pk.to_pub64(&mut ctx);
			assert_eq!(
				secp256k1_musig_partial_sig_verify(
					ctx.secp(),
					&slate_userb.participant_data[1].part_sigs[1] as *const u8,
					&slate_userb.participant_data[1].public_nonces[1] as *const u8,
					&pk as *const u8,
					&slate_userb.session.keyagg_cache as *const u8,
					&slate_userb.session.session as *const u8
				),
				1
			);
		}

		let slate_ser = slate_userb.serialize().unwrap();
		let mut slate_usera = Slate::deserialize(slate_ser).unwrap();

		unsafe {
			let pk = PublicKey::from(&mut ctx, offset).to_pub64(&mut ctx);
			assert_eq!(
				secp256k1_musig_partial_sig_verify(
					ctx.secp(),
					&slate_usera.participant_data[1].part_sigs[0] as *const u8,
					&slate_usera.participant_data[1].public_nonces[0] as *const u8,
					&pk as *const u8,
					&slate_usera.session.keyagg_cache as *const u8,
					&slate_usera.session.session as *const u8
				),
				1
			);
			let negavite_pkoutput = pkoutput.negate(&mut ctx).unwrap();
			let pk = PublicKey::from(&mut ctx, negavite_pkoutput);
			let pk = pk.to_pub64(&mut ctx);
			assert_eq!(
				secp256k1_musig_partial_sig_verify(
					ctx.secp(),
					&slate_usera.participant_data[1].part_sigs[1] as *const u8,
					&slate_usera.participant_data[1].public_nonces[1] as *const u8,
					&pk as *const u8,
					&slate_usera.session.keyagg_cache as *const u8,
					&slate_usera.session.session as *const u8
				),
				1
			);

			slate_usera.sec_nonces = slate.sec_nonces;

			assert!(slate_usera
				.sign_index(0, &mut ctx, vec![pkinput].unwrap(), Vec::new())
				.is_ok());

			let pk = PublicKey::from(&mut ctx, pkinput).to_pub64(&mut ctx);
			assert_eq!(
				secp256k1_musig_partial_sig_verify(
					ctx.secp(),
					&slate_usera.participant_data[0].part_sigs[0] as *const u8,
					&slate_usera.participant_data[0].public_nonces[0] as *const u8,
					&pk as *const u8,
					&slate_usera.session.keyagg_cache as *const u8,
					&slate_usera.session.session as *const u8
				),
				1
			);
		}

		assert_eq!(slate_usera.participant_data[0].part_sigs.len(), 1);
		assert_eq!(slate_usera.participant_data[1].part_sigs.len(), 2);

		assert!(slate_usera.finalize(&mut ctx).is_ok());

		let public_blind_excess_sum = slate_usera.public_blind_sum(&mut ctx).unwrap();
		let mut sha3_256 = SHA3_256::new();
		let mut feebytes = [0u8; 8];
		to_le_bytes_u64(slate_usera.fee, &mut feebytes);
		sha3_256.update(feebytes);
		sha3_256.update(public_blind_excess_sum.as_ref());
		let msg: [u8; 32] = sha3_256.finalize();
		let agg_pk = slate_usera.public_blind_sum(&mut ctx).unwrap();

		/*
		unsafe {
			assert_eq!(
				secp256k1_schnorrsig_verify(
					ctx.secp(),
					&slate_usera.final_sig.unwrap() as *const u8,
					&msg as *const u8,
					32,
					&agg_pk as *const u8
				),
				1
			);
		}
				*/
	}
}
