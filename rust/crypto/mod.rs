/*
https://github.com/pornin/crrl
MIT License

Copyright (c) 2022 Thomas Pornin

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

#[cfg(all(feature = "alloc", not(feature = "std")))]
#[macro_use]
#[allow(unused_imports)]
//extern crate alloc;
#[cfg(feature = "std")]
#[macro_use]
#[allow(unused_imports)]
//extern crate std;
#[cfg(all(feature = "alloc", not(feature = "std")))]
#[allow(unused_imports)]
//pub(crate) use alloc::vec::Vec;
#[cfg(feature = "std")]
#[allow(unused_imports)]
//pub(crate) use std::vec::Vec;

/// The `rand_core` types are re-exported so that users of crrl do not
/// have to worry about using the exact correct version of `rand_core`.
//pub use rand_core::{CryptoRng, RngCore, Error as RngError};

#[allow(unused_macros)]
macro_rules! static_assert {
	($condition:expr) => {
		let _ = &[()][1 - ($condition) as usize];
	};
}

//pub mod backend;
//pub mod field;

//pub use self::backend::{Zu128, Zu256, Zu384};

#[cfg(feature = "ed25519")]
//pub mod ed25519;
#[cfg(feature = "x25519")]
//pub mod x25519;
#[cfg(feature = "ristretto255")]
//pub mod ristretto255;
#[cfg(feature = "jq255e")]
//pub mod jq255e;
#[cfg(feature = "jq255s")]
//pub mod jq255s;
#[cfg(feature = "p256")]
//pub mod p256;
#[cfg(feature = "secp256k1")]
//pub mod secp256k1;
#[cfg(feature = "gls254")]
//pub mod gls254;
#[cfg(feature = "ed448")]
pub mod ed448;
#[cfg(feature = "x448")]
pub mod x448;

#[cfg(feature = "decaf448")]
//pub mod decaf448;
#[cfg(all(feature = "alloc", feature = "frost"))]
//pub mod frost;
#[cfg(feature = "lms")]
//pub mod lms;
#[cfg(feature = "blake2s")]
//pub mod blake2s;
pub mod sha2;
//pub mod sha3;
