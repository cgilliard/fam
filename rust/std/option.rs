#[derive(PartialEq, Debug)]
pub enum Option<T> {
	None,
	Some(T),
}
