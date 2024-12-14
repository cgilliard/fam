use core::marker::Sized;
use std::boxed::Box;
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

macro_rules! impl_clone_for_box {
    ($($t:ty),*) => {
        $(
            impl Clone for Box<$t> {
                fn clone(&self) -> Result<Self, Error> {
                    Box::new(*self.as_ref())
                }
            }
        )*
    };
}

// Use the macro to generate the `Clone` impl for each primitive type
impl_clone_for_box!(i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, f32, f64, bool);

#[cfg(test)]
mod test {
	use super::Clone;
	use err;
	use std::error::{Error, ErrorKind::Alloc};
	use std::result::{Result, Result::Err, Result::Ok};

	struct X {
		x: u32,
		y: u64,
	}

	impl Clone for X {
		fn clone(&self) -> Result<X, Error> {
			if self.x == 100 {
				// simulate err
				Err(err!(Alloc))
			} else {
				Ok(Self {
					x: self.x,
					y: self.y,
				})
			}
		}
	}

	#[test]
	fn test_clone() {
		let x = X { x: 1, y: 2 };
		let yp = x.clone();
		assert!(!yp.is_err());
		let y = yp.unwrap();
		assert_eq!(y.x, 1);
		assert_eq!(y.y, 2);
		let mut z = X { x: 10, y: 20 };
		assert_eq!(z.x, 10);
		assert_eq!(z.y, 20);
		z.clone_from(&x);
		assert_eq!(z.x, 1);
		assert_eq!(z.y, 2);
		let a = X { x: 100, y: 20 };
		let mut e = X { x: 1, y: 0 };
		let res = e.clone_from(&a);
		assert!(res.is_err());
	}
}
