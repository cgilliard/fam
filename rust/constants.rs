#![allow(dead_code)]

pub const SECP256K1_START_NONE: u32 = (1 << 0) | 0;
pub const SECP256K1_START_VERIFY: u32 = (1 << 0) | (1 << 8);
pub const SECP256K1_START_SIGN: u32 = (1 << 0) | (1 << 9);
pub const SECP256K1_SER_UNCOMPRESSED: u32 = (1 << 1) | 0;
pub const SECP256K1_SER_COMPRESSED: u32 = (1 << 1) | (1 << 8);
