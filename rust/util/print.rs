#[macro_export]
macro_rules! println {
	($i:expr) => {
		write(2, "test\n".as_ptr(), 5);
	};
}

pub trait Printable {
	fn print(self);
}
