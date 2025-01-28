use crypto::context::Context;
use crypto::ffi::cpsrng_rand_bytes_ctx;

#[derive(Clone, Copy)]
pub struct Session {
	keyagg_cache: [u8; 256],
	session: [u8; 256],
	session_id: [u8; 32],
}

impl Session {
	pub fn new(ctx: &mut Context) -> Self {
		let keyagg_cache = [0u8; 256];
		let session = [0u8; 256];
		Self::from_parts(ctx, keyagg_cache, session)
	}

	pub fn from_parts(ctx: &mut Context, keyagg_cache: [u8; 256], session: [u8; 256]) -> Self {
		let mut session_id = [0u8; 32];

		unsafe {
			cpsrng_rand_bytes_ctx(ctx.rand(), &mut session_id as *mut u8, 32);
		}
		Self {
			keyagg_cache,
			session,
			session_id,
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
