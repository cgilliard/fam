use core::ptr::copy_nonoverlapping;
use crypto::sha3::SHA3_512;
use prelude::*;

#[derive(Clone, Copy)]
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
