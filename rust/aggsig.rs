#![allow(dead_code)]
#![allow(unused_variables)]

use prelude::*;

pub struct SecretKey([u8; 64]);
pub struct PublicKey([u8; 32]);

pub fn calculate_partial_sig(
	ctx: *mut u8,
	sec_key: &SecretKey,
	sec_nonce: &SecretKey,
	nonce_sum: &PublicKey,
	pubkey_sum: Option<&PublicKey>,
	msg: [u8; 32],
) -> Result<[u8; 64], Error> {
	let v = [0u8; 64];
	Ok(v)
}
