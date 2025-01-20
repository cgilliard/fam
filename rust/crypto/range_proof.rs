use crypto::ed448::PrivateKey;
use crypto::pedersen::Commitment;

pub struct RangeProof([u8; 1992]);
// structure:
// [56 bytes - commitment]
// [Scalar values - 56 byte X 16]
// [Challenge scalar - 56 byte]
// [Transcript - sha3 hash - 32 bytes]
// [Inner Product Argument - 952 bytes]
// Total size = 1992 bytes (max before compaction)

impl RangeProof {
	pub fn prove(_c: Commitment) -> Self {
		let v = [0u8; 1992];
		RangeProof(v)
	}

	pub fn verify(_proof: Self) -> bool {
		false
	}

	// return (r / v)
	pub fn rewind(_proof: Self, _key: PrivateKey) -> (u64, u64) {
		(0, 0)
	}
}
