use core::mem::{drop, size_of};
use core::ops::Drop;
use core::ptr;
use err;
use std::error::{Error, ErrorKind::Alloc};
use std::result::{Result, Result::Err, Result::Ok};
use sys::{map, unmap};

pub struct Box<T> {
	value: *mut T,
	pages: usize,
}

impl<T> Drop for Box<T> {
	fn drop(&mut self) {
		unsafe {
			let value = ptr::read(self.value);
			drop(value);
			unmap(self.value as *mut u8, self.pages);
		}
	}
}

impl<T> Box<T> {
	pub fn new(t: T) -> Result<Self, Error> {
		let pages = pages!(size_of::<T>());
		let value;
		unsafe {
			value = map(pages) as *mut T;
			if value.is_null() {
				return Err(err!(Alloc));
			}
			ptr::write(value, t);
		}

		Ok(Self {
			pages: pages as usize,
			value,
		})
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
	#[test]
	fn test_box() {
		let mut x = Box::new(4).unwrap();
		let y = x.as_ref();
		assert_eq!(*y, 4);

		let z = x.as_mut();
		*z = 10;
		assert_eq!(*z, 10);
	}
}
