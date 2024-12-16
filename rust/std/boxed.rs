use core::marker::Sized;
use core::mem::size_of;
use core::ops::Deref;
use core::ops::Drop;
use core::ptr;
use err;
use std::clone::Clone;
use std::error::{Error, ErrorKind::Alloc};
use std::result::{Result, Result::Err, Result::Ok};
use sys::{map, unmap};

pub struct Box<T: ?Sized> {
	pub value: *mut T,
	pub pages: usize,
	pub leak: bool,
}

impl<T: ?Sized> Drop for Box<T> {
	fn drop(&mut self) {
		if !self.leak {
			let pages = self.pages;

			unsafe {
				ptr::drop_in_place(self.value);
				unmap(self.value as *mut u8, pages);
			}
		}
	}
}

impl<T: Clone> Clone for Box<T> {
	fn clone(&self) -> Result<Self, Error> {
		let value = self.as_ref();
		match value.clone() {
			Ok(v) => Box::new(v),
			Err(e) => Err(e),
		}
	}
}

impl<T> Deref for Box<T>
where
	T: ?Sized,
{
	type Target = T;

	fn deref(&self) -> &Self::Target {
		unsafe { &*self.value }
	}
}

impl<T> Box<T> {
	pub fn new(t: T) -> Result<Self, Error> {
		let size = size_of::<T>();
		let leak = false;
		let value;
		let pages = pages!(size);
		unsafe {
			value = map(pages) as *mut T;
			if value.is_null() {
				return Err(err!(Alloc));
			}
			ptr::write(value, t);
		}

		Ok(Self { value, pages, leak })
	}

	pub fn value(&self) -> *mut T {
		self.value
	}

	pub fn pages(&self) -> usize {
		self.pages
	}

	pub unsafe fn leak(&mut self) {
		self.leak = true;
	}

	pub fn as_ref(&self) -> &T {
		unsafe { &*self.value }
	}

	pub fn as_mut(&mut self) -> &mut T {
		unsafe { &mut *self.value }
	}
}

#[macro_export]
macro_rules! box_dyn {
	($type:expr, $trait:ident) => {{
		match Box::new($type) {
			Ok(mut boxv) => {
				unsafe {
					boxv.leak();
				}
				let boxv_dyn: Box<dyn $trait> = Box {
					value: boxv.value(),
					pages: boxv.pages(),
					leak: false,
				};
				Ok(boxv_dyn)
			}
			Err(e) => Err(e),
		}
	}};
}

#[cfg(test)]
mod test {
	use std::boxed::Box;
	use std::clone::Clone;
	use std::result::{Result::Err, Result::Ok};

	#[test]
	fn test_box() {
		let mut x = Box::new(4).unwrap();
		let y = x.as_ref();
		assert_eq!(*y, 4);

		let z = x.as_mut();
		*z = 10;
		assert_eq!(*z, 10);
		let a = x.clone().unwrap();
		let b = a.as_ref();
		assert_eq!(*b, 10);
	}

	trait GetData {
		fn get_data(&self) -> i32;
	}

	struct TestSample {
		data: i32,
	}

	impl GetData for TestSample {
		fn get_data(&self) -> i32 {
			self.data
		}
	}

	#[test]
	fn test_dyn() {
		let t = TestSample { data: 1 };
		let mut sample = Box::new(t).unwrap();
		unsafe {
			sample.leak();
		}
		let sample_b: Box<dyn GetData> = Box {
			value: sample.value,
			pages: sample.pages,
			leak: false,
		};

		assert_eq!(sample_b.get_data(), 1);

		// create a dynamic dispatch to trait GetData
		let v: Box<dyn GetData> = match box_dyn!(TestSample { data: 2 }, GetData) {
			Ok(v) => v,
			Err(_e) => exit!("box_dyn failed!"),
		};
		assert_eq!(v.get_data(), 2);
	}
}
