#[must_use = "This `Result` must be used, or explicitly handled with `unwrap`, `is_err`, or similar."]
#[derive(PartialEq, Debug)]
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

#[cfg(test)]
mod test {
	use prelude::*;

	fn test_result() -> Result<(), Error> {
		let x: Result<u32, Error> = Ok(1u32);
		let y = x.unwrap();
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
