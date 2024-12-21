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
