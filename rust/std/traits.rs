pub trait Hash {
	fn hash(&self) -> usize;
}

pub trait Equal {
	fn equal(&self, other: &Self) -> bool;
}

pub enum Ordering {
	Less = -1,
	Equal = 0,
	Greater = 1,
}

/*
impl Equal for Ordering {
	fn equal(&self, other
}
*/

pub trait Ord {
	fn compare(&self, other: &Self) -> i8;
}

impl Ord for i32 {
	fn compare(&self, other: &Self) -> i8 {
		if *self < *other {
			-1
		} else if *self > *other {
			1
		} else {
			0
		}
	}
}

impl Ord for u64 {
	fn compare(&self, other: &Self) -> i8 {
		if *self < *other {
			-1
		} else if *self > *other {
			1
		} else {
			0
		}
	}
}
