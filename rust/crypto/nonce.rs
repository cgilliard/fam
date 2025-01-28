use crypto::context::Context;
use crypto::ffi::cpsrng_rand_bytes_ctx;
use prelude::*;

#[derive(Clone, Copy)]
pub struct SecNonce([u8; 132]);

impl AsRef<[u8]> for SecNonce {
	fn as_ref(&self) -> &[u8] {
		&self.0
	}
}

impl SecNonce {
	pub fn new(ctx: &mut Context) -> Self {
		let mut v = [0u8; 132];
		unsafe {
			cpsrng_rand_bytes_ctx(ctx.rand(), &mut v as *mut u8, 132);
		}
		Self(v)
	}
}
