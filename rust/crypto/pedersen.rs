use core::convert::AsRef;
use core::ptr::copy_nonoverlapping;
use crypto::ed448::{Point, PrivateKey, Scalar};
use prelude::*;

pub struct Commitment([u8; 57]);

impl AsRef<[u8]> for Commitment {
	fn as_ref(&self) -> &[u8] {
		&self.0
	}
}

impl Clone for Commitment {
	fn clone(&self) -> Result<Self, Error> {
		let mut value = [0u8; 57];
		unsafe {
			copy_nonoverlapping(self.0.as_ptr(), value.as_mut_ptr(), 57);
		}
		Ok(Commitment(value))
	}
}

impl Commitment {
	// for testing purposes we have non-random blinding factors
	#[cfg(test)]
	pub fn from_parts(r: u64, v: u64) -> Self {
		let r_scalar = Scalar::from_u64(r);
		let v_scalar = Scalar::from_u64(v);
		let r_G = Point::BASE * r_scalar;
		let v_H = Point::derive_h() * v_scalar;
		let point = v_H + r_G;
		Commitment(point.encode())
	}

	pub fn generate(v: u64) -> (Self, PrivateKey) {
		let k = PrivateKey::generate();
		let v_scalar = Scalar::from_u64(v);
		let r_scalar = Scalar::w8le(k.encode());
		let r_G = Point::BASE * r_scalar;
		let v_H = Point::derive_h() * v_scalar;
		let point = v_H + r_G;
		(Commitment(point.encode()), k)
	}

	pub fn generate_448(v_scalar: &Scalar) -> (Self, PrivateKey) {
		let k = PrivateKey::generate();
		let r_scalar = Scalar::w8le(k.encode());
		let r_G = Point::BASE * r_scalar;
		let v_H = Point::derive_h() * v_scalar;
		let point = v_H + r_G;
		(Commitment(point.encode()), k)
	}

	pub fn commit_to_vector(vector: &Vec<Scalar>) -> (Self, PrivateKey) {
		let k = PrivateKey::generate();
		let r_scalar = Scalar::w8le(k.encode());
		let mut commitment = Point::NEUTRAL;
		for scalar in vector {
			commitment = commitment + (Point::derive_h() * scalar);
		}
		commitment = commitment + (Point::BASE * r_scalar); // Add blinding factor
		(Commitment(commitment.encode()), k)
	}
}
