use crypto::ed448::{Point, Scalar};

pub struct Commitment([u8; 57]);

impl Commitment {
	pub fn from_parts(r: u64, v: u64) -> Self {
		let r_scalar = Scalar::from_u64(r);
		let v_scalar = Scalar::from_u64(v);
		let r_G = Point::BASE * r_scalar;
		let v_H = Point::derive_h() * v_scalar;
		let point = v_H + r_G;
		Commitment(point.encode())
	}
}
