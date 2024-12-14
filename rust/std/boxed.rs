use core::mem::{drop, size_of};
use core::ops::Drop;
use core::ptr;
use core::ptr::copy_nonoverlapping;
use err;
use std::clone::Clone;
use std::error::{Error, ErrorKind::Alloc};
use std::result::{Result, Result::Err, Result::Ok};
use sys::{map, unmap};

pub struct Box<T> {
	value: *mut T,
}

impl<T> Drop for Box<T> {
	fn drop(&mut self) {
		let pages = pages!(size_of::<T>());
		unsafe {
			let value = ptr::read(self.value);
			drop(value);
			unmap(self.value as *mut u8, pages);
		}
	}
}

impl<T> Clone for Box<T> {
	fn clone(&self) -> Result<Self, Error> {
		let value = unsafe { &mut *self.value };
		let nvalue;
		let pages = pages!(size_of::<T>());
		unsafe {
			nvalue = map(pages) as *mut T;
			if nvalue.is_null() {
				return Err(err!(Alloc));
			}
			copy_nonoverlapping(value, nvalue, size_of::<T>());
		}

		Ok(Self { value: nvalue })
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

		Ok(Self { value })
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
	}
}
