#[derive(Clone, Copy)]
pub struct Session {
	pub keyagg_cache: [u8; 256],
	pub session: [u8; 256],
}

impl Session {
	pub fn new() -> Self {
		let keyagg_cache = [0u8; 256];
		let session = [0u8; 256];
		Self::from_parts(keyagg_cache, session)
	}

	pub fn from_parts(keyagg_cache: [u8; 256], session: [u8; 256]) -> Self {
		Self {
			keyagg_cache,
			session,
		}
	}
}
