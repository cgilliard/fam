use core::convert::AsRef;
use crypto::ed448::PrivateKey;
use crypto::ed448::Scalar;
use crypto::pedersen::Commitment;
use crypto::sha3::SHA3_512;
use prelude::*;

pub struct RangeProof {
	c_reduced: Vec<Commitment>,
	a_final: Scalar,
	b_final: Scalar,
	challenges: Vec<[u8; 57]>,
	final_commitment: Commitment,
	s_L: Scalar,
	s_R: Scalar,
	t_1: Scalar,
	t_2: Scalar,
}

fn calculate_s_L(a_reduced: &Vec<Scalar>, challenges: &Vec<[u8; 57]>) -> Scalar {
	let mut s_L = Scalar::ZERO;
	let mut x_inverse = Scalar::ONE; // Start with x^0

	for i in (0..challenges.len()).rev() {
		let challenge = Scalar::w8le(challenges[i]);
		let x = challenge.invert();
		x_inverse *= x;

		if i < a_reduced.len() {
			s_L += x_inverse * a_reduced[i];
		}
	}
	s_L
}

fn calculate_s_R(b_reduced: &Vec<Scalar>, challenges: &Vec<[u8; 57]>) -> Scalar {
	let mut s_R = Scalar::ZERO;
	let mut x = Scalar::ONE; // Start with x^0

	for i in 0..challenges.len() {
		let challenge = Scalar::w8le(challenges[i]);
		x *= challenge;

		if i < b_reduced.len() {
			s_R += x * b_reduced[i];
		}
	}

	s_R
}

fn calculate_t_1(challenges: &Vec<[u8; 57]>, y: Scalar, z: Scalar) -> Scalar {
	let mut t_1 = Scalar::ZERO;
	let mut x = Scalar::ONE;

	for i in 0..challenges.len() {
		let challenge = Scalar::w8le(challenges[i]);
		x *= challenge;

		let mut exp = [0u64; 7];
		exp[0] = i as u64;
		let mut exp_i = Scalar::from_u64(2);
		exp_i.set_modpow_pubexp(&exp);

		t_1 += x * (z - z * z) * exp_i;

		// Calculate exp_2i separately
		exp[0] = 2 * i as u64;
		let mut exp_2i = Scalar::from_u64(2);
		exp_2i.set_modpow_pubexp(&exp);
		t_1 += x * x * y * y * exp_2i;
	}

	t_1
}

fn calculate_t_2(challenges: &Vec<[u8; 57]>, z: Scalar) -> Scalar {
	let mut t_2 = Scalar::ZERO;
	let mut x = Scalar::ONE;

	for i in 0..challenges.len() {
		let challenge = Scalar::w8le(challenges[i]);
		x *= challenge;

		let mut exp = [0u64; 7];
		exp[0] = i as u64;
		let mut exp_i = Scalar::from_u64(2);
		exp_i.set_modpow_pubexp(&exp);

		t_2 += x * z * z * z * exp_i;
	}

	t_2
}

/*
fn calculate_t_2(challenges: &Vec<[u8; 57]>, z: Scalar) -> Scalar {
	let mut t_2 = Scalar::ZERO;
	let mut x = Scalar::ONE;

	for (i, challenge_bytes) in challenges.iter().enumerate() {
		let challenge = Scalar::w8le(*challenge_bytes);
		x *= challenge;

		t_2 += x * z * z * z * Scalar::from(2).pow([i as u64]);
	}

	t_2
}
*/

impl RangeProof {
	pub fn prove(c: &Commitment, v: u64) -> Result<Self, Error> {
		let mut a = Vec::new();
		let mut b = Vec::new();
		match a.resize(64) {
			Ok(_) => {}
			Err(e) => return Err(e),
		}
		match b.resize(64) {
			Ok(_) => {}
			Err(e) => return Err(e),
		}

		for i in 0..64 {
			if (v >> i) & 1 == 1 {
				a[i] = Scalar::ONE;
			} else {
				a[i] = Scalar::ZERO;
			}
			b[i] = Scalar::from_u64(1u64 << i);
		}

		let mut inner_product = Scalar::ZERO;
		for i in 0..64 {
			let product = a[i] * b[i];
			inner_product += product;
		}

		let (inner_commit, _ikey) = Commitment::generate_448(&inner_product);
		let (a_commit, _akey) = Commitment::commit_to_vector(&a);
		let (b_commit, _bkey) = Commitment::commit_to_vector(&b);

		let mut a_reduced = a.clone().unwrap();
		let mut b_reduced = b.clone().unwrap();

		let mut c_reduced = vec![inner_commit].unwrap();
		let mut challenges = Vec::new();

		while a_reduced.len() > 1 {
			let len: usize = a_reduced.len();

			let a_hi = &a_reduced[0..len / 2];
			let a_low = &a_reduced[len / 2..len];

			let b_hi = &b_reduced[0..len / 2];
			let b_low = &b_reduced[len / 2..len];

			let mut a_reduced_new = Vec::new();
			let mut b_reduced_new = Vec::new();

			for i in 0..len / 2 {
				match a_reduced_new.push((a_low[i] + a_hi[i]).half()) {
					Ok(_) => {}
					Err(e) => return Err(e),
				}
				match b_reduced_new.push((b_low[i] + b_hi[i]).half()) {
					Ok(_) => {}
					Err(e) => return Err(e),
				}
			}

			let (a_commit_reduced, _) = Commitment::commit_to_vector(&a_reduced_new);
			let (b_commit_reduced, _) = Commitment::commit_to_vector(&b_reduced_new);

			let mut sha512 = SHA3_512::new();
			sha512.update(a_commit_reduced.as_ref());
			sha512.update(b_commit_reduced.as_ref());
			let hash_finalize = sha512.finalize();
			let mut hash = [0u8; 57];
			for i in 0..57 {
				hash[i] = hash_finalize[i];
			}
			let hash_scalar = Scalar::w8lev(&hash);

			let mut new_inner_product = Scalar::ZERO;
			for i in 0..a_low.len() {
				new_inner_product += a_low[i] * b_hi[i]
					+ hash_scalar * a_low[i] * b_low[i]
					+ hash_scalar * hash_scalar * a_hi[i] * b_hi[i]
					+ hash_scalar * hash_scalar * hash_scalar * a_hi[i] * b_low[i];
			}

			let (inner_commit_reduced, _) = Commitment::generate_448(&new_inner_product);
			match c_reduced.push(inner_commit_reduced) {
				Ok(_) => {}
				Err(e) => return Err(e),
			}

			match challenges.push(hash) {
				Ok(_) => {}
				Err(e) => return Err(e),
			}

			a_reduced = a_reduced_new;
			b_reduced = b_reduced_new;
		}

		let a_final = a_reduced[0];
		let b_final = b_reduced[0];
		let final_commitment = c_reduced[c_reduced.len() - 1].clone().unwrap();
		let s_L = calculate_s_L(&a_reduced, &challenges);
		let s_R = calculate_s_R(&b_reduced, &challenges);

		// Generate y (using Fiat-Shamir)
		let mut hasher = SHA3_512::new();
		hasher.update(a_commit.as_ref());
		hasher.update(b_commit.as_ref());
		hasher.update(c.as_ref());
		let hash_y = hasher.finalize();
		let y = Scalar::w8lev(&hash_y);

		// Calculate z (using Fiat-Shamir)
		let mut hasher = SHA3_512::new();
		hasher.update(hash_y.as_ref());
		let hash_z = hasher.finalize();
		let z = Scalar::w8lev(&hash_z);

		let t_1 = calculate_t_1(&challenges, y, z);
		let t_2 = calculate_t_2(&challenges, z);

		Ok(Self {
			a_final,
			b_final,
			final_commitment,
			c_reduced,
			challenges,
			s_L,
			s_R,
			t_1,
			t_2,
		})
	}

	pub fn verify(_proof: Self) -> bool {
		false
	}

	// return (r / v)
	pub fn rewind(_proof: Self, _key: PrivateKey) -> Option<(u64, u64)> {
		None
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn test_proof() {
		let c = Commitment::from_parts(1234, 0xF);
		let _key = PrivateKey::generate();
		let p1 = RangeProof::prove(&c, 0xF);
		assert!(p1.is_ok());
		/*
		for i in 0..p1.as_ref().len() {
			//println!("v[{}] = {}", i, p1.as_ref()[i]);
		}
				*/
	}
}
