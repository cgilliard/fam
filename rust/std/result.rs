use core::convert::Infallible;
use core::ops::{ControlFlow, FromResidual, Try};
use prelude::*;

pub enum Result<T, E> {
	Ok(T),
	Err(E),
}

impl<T, E> Result<T, E> {
	pub fn unwrap(self) -> T {
		match self {
			Result::Ok(t) => t,
			Result::Err(_e) => exit!("unwrap on error!"),
		}
	}

	pub fn is_err(&self) -> bool {
		match self {
			Result::Ok(_) => false,
			_ => true,
		}
	}

	pub fn is_ok(&self) -> bool {
		!self.is_err()
	}
}

impl<T, E> Try for Result<T, E> {
	type Output = T;
	type Residual = Result<Infallible, E>;

	fn from_output(output: Self::Output) -> Self {
		Ok(output)
	}

	fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
		match self {
			Ok(v) => ControlFlow::Continue(v),
			Err(e) => ControlFlow::Break(Err(e)),
		}
	}
}

impl<T, E, F: From<E>> FromResidual<Result<Infallible, E>> for Result<T, F> {
	fn from_residual(residual: Result<Infallible, E>) -> Self {
		match residual {
			Err(e) => Err(From::from(e)),
		}
	}
}

#[cfg(test)]
mod test {
	use prelude::*;

	fn test_result() -> Result<(), Error> {
		let x: Result<u32, Error> = Ok(1u32);
		let y = x?;
		assert_eq!(y, 1);

		Ok(())
	}

	#[test]
	fn call_test_result() {
		match test_result() {
			Ok(_) => {}
			Err(_e) => {
				println!("Error!");
				assert!(false);
			}
		}
	}
}
