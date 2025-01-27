use crypto::keys::PublicKey;
use crypto::session::Session;
use prelude::*;

#[derive(Copy, Clone)]
pub struct Signature([u8; 64]);

pub struct SlateContext {
	_session_id: [u8; 32],
	_secnonce: [u8; 132],
}

pub struct ParticipantData {
	pub public_blind_excess: PublicKey,
	pub public_nonce: [u8; 132],
	pub part_sig: Option<Signature>,
}

pub struct Slate {
	_session: Session,
	_participant_data: Vec<ParticipantData>,
}

impl AsRef<[u8]> for Signature {
	fn as_ref(&self) -> &[u8] {
		&self.0
	}
}
