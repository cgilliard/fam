#[derive(PartialEq, Debug)]
pub enum Option<T> {
	None,
	Some(T),
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
}
