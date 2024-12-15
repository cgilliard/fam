use core::mem;
use core::mem::drop;
use core::mem::size_of;
use core::ops::Drop;
use core::ptr;
use err;
use std::clone::Clone;
use std::error::{Error, ErrorKind::Alloc};
use std::option::{Option, Option::None, Option::Some};
use std::result::{Result, Result::Err, Result::Ok};
use sys::{map, unmap};

pub struct Rc<T> {
	inner: *mut RcInner<T>,
}

struct RcInner<T> {
	count: u64,
	value: T,
}

impl<T> Clone for Rc<T> {
	fn clone(&self) -> Result<Self, Error> {
		let inner = unsafe { &mut *self.inner };
		aadd!(&mut inner.count, 1);
		Ok(Self { inner: self.inner })
	}
}

impl<T> Drop for Rc<T> {
	fn drop(&mut self) {
		let inner = unsafe { &mut *self.inner };
		if asub!(&mut inner.count, 1) == 1 {
			let pages = pages!(size_of::<T>());
			unsafe {
				let value = mem::replace(&mut inner.value, mem::zeroed());
				drop(value);
				unmap(self.inner as *mut u8, pages);
			}
		}
	}
}

impl<T> Rc<T> {
	pub fn new(value: T) -> Result<Self, Error> {
		let pages = pages!(size_of::<T>());
		let inner;
		unsafe {
			inner = map(pages) as *mut RcInner<T>;
			if inner.is_null() {
				return Err(err!(Alloc));
			}
			ptr::write(inner, RcInner { count: 1, value });
		}

		Ok(Self { inner })
	}

	pub fn get(&self) -> &T {
		unsafe { &(&*self.inner).value }
	}

	pub fn get_mut(&mut self) -> Option<&mut T> {
		if aload!(&mut (*self.inner).count) == 1 {
			unsafe { Some(&mut (&mut *self.inner).value) }
		} else {
			None
		}
	}

	pub unsafe fn get_mut_unchecked(&mut self) -> &mut T {
		unsafe { &mut (&mut *self.inner).value }
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use std::string::String;

	#[test]
	fn test_rc() {
		let mut x1 = Rc::new(1).unwrap();
		assert!(x1.get_mut().is_some());
		let mut x2 = x1.clone().unwrap();
		assert!(x1.get_mut().is_none());
		assert!(x2.get_mut().is_none());

		unsafe {
			*x1.get_mut_unchecked() += 1;
		}
		assert_eq!(*x1.get(), 2);
		assert_eq!(*x2.get(), 2);

		let s = String::new("0123456789hi0123456789hi0123456789hi0123456789hi0123456789").unwrap();
		let y1 = Rc::new(s).unwrap();
		let y2 = y1.clone().unwrap();
		let y3 = y2.clone().unwrap();
		assert_eq!(y3.get().len(), 58);
		let y3 = y2.clone().unwrap();
		assert_eq!(y1.get().len(), 58);
		assert_eq!(y2.get().len(), 58);
		assert_eq!(y3.get().len(), 58);
	}
}
