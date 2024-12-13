pub enum Result<T, E> {
	Ok(T),
	Err(E),
}

impl<T, E> Result<T, E> {
	pub fn unwrap(self) -> T {
		match self {
			Result::Ok(t) => t,
			Result::Err(_e) => {
				panic!("unwrap on error!");
			}
		}
	}
}
