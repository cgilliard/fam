use std::clone::Clone;
use std::error::Error;
use std::option::Option::{None, Some};
use std::result::{Result, Result::Err, Result::Ok};

#[derive(PartialEq, Debug)]
pub enum Option<T> {
	None,
	Some(T),
}

impl<T> Clone for Option<T>
where
	T: Clone,
{
	fn clone(&self) -> Result<Self, Error> {
		match self {
			Some(v) => match v.clone() {
				Ok(v) => Ok(Some(v)),
				Err(e) => Err(e),
			},
			None => Ok(None),
		}
	}
}

impl<T> Option<T> {
	pub fn is_some(&self) -> bool {
		match self {
			Option::Some(_) => true,
			_ => false,
		}
	}
	pub fn is_none(&self) -> bool {
		!self.is_some()
	}

	pub const fn as_mut(&mut self) -> Option<&mut T> {
		match self {
			Some(v) => Some(v),
			None => None,
		}
	}

	pub const fn as_ref(&self) -> Option<&T> {
		match self {
			Some(v) => Some(v),
			None => None,
		}
	}
}
