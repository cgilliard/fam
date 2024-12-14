use core::mem::size_of;
use core::ops::Drop;
use core::ptr;
use err;
use std::clone::Clone;
use std::error::{Error, ErrorKind::Alloc};
use std::result::{Result, Result::Err, Result::Ok};
use sys::{map, unmap};

pub struct Box<T> {
	value: *mut T,
	pages: usize,
}

impl<T> Drop for Box<T> {
	fn drop(&mut self) {
		let pages = self.pages;

		unsafe {
			ptr::drop_in_place(self.value);
			unmap(self.value as *mut u8, pages);
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

impl<T> Box<T> {
	pub fn new(t: T) -> Result<Self, Error> {
		let value;
		let pages = pages!(size_of::<T>());
		unsafe {
			value = map(pages) as *mut T;
			if value.is_null() {
				return Err(err!(Alloc));
			}
			ptr::write(value, t);
		}

		Ok(Self { value, pages })
	}

	pub fn as_ref(&self) -> &T {
		unsafe { &*self.value }
	}

	pub fn as_mut(&mut self) -> &mut T {
		unsafe { &mut *self.value }
	}
}

#[cfg(test)]
mod test {
	use std::boxed::Box;
	use std::clone::Clone;

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

		//let bt = BoxTest { v };
		/*
		let v = BoxTest {
			v: Box::new([0u8; 5]).unwrap().into(),
		};
				*/

		//let v2 = v.v.as_ref();
	}
}
