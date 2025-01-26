use crypto::ffi::*;
use prelude::*;
use sys::safe_release;

pub struct Context {
	secp: *mut u8,
	rand: *mut u8,
}

impl Drop for Context {
	fn drop(&mut self) {
		unsafe {
			secp256k1_context_destroy(self.secp);
			safe_release(self.rand);
		}
	}
}

impl Context {
	pub fn new() -> Result<Self, Error> {
		let secp = unsafe { secp256k1_context_create(SECP256K1_CONTEXT_NONE) };
		if secp.is_null() {
			Err(err!(SecpInit))
		} else {
			let rand = unsafe { cpsrng_context_create() };
			if rand.is_null() {
				unsafe {
					secp256k1_context_destroy(secp);
				}
				Err(err!(SecpInit))
			} else {
				let mut r = [0u8; 32];
				unsafe {
					cpsrng_rand_bytes_ctx(rand, &mut r as *mut u8, 32);
					secp256k1_context_randomize(secp, &r as *const u8);
				}
				Ok(Self { secp, rand })
			}
		}
	}

	pub fn secp(&self) -> *mut u8 {
		self.secp
	}

	pub fn rand(&self) -> *mut u8 {
		self.rand
	}
}
