use core::marker::Sized;
use std::error::Error;
use std::result::{Result, Result::Err, Result::Ok};

pub trait Clone: Sized {
	fn clone(&self) -> Result<Self, Error>;
	fn clone_from(&mut self, source: &Self) -> Result<(), Error> {
		let src = source.clone();
		match src {
			Ok(src) => {
				*self = src;
				Ok(())
			}
			Err(e) => Err(e),
		}
	}
}
