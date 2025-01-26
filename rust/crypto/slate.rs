use crypto::keys::PublicKey;
use crypto::session::Session;
use prelude::*;

#[derive(Copy, Clone)]
pub struct Signature([u8; 64]);

pub struct SlateContext {
	session_id: [u8; 32],
	secnonce: [u8; 132],
}

pub struct ParticipantData {
	pub public_blind_excess: PublicKey,
	pub public_nonce: [u8; 132],
	pub part_sig: Option<Signature>,
}

pub struct Slate {
	session: Session,
	participant_data: Vec<ParticipantData>,
}
