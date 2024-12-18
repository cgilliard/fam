use crate::*;
use core::ops::{Deref, DerefMut, Drop};

struct RcInner<T> {
	count: u64,
	value: T,
}

pub struct Rc<T> {
	inner: Box<RcInner<T>>,
}

impl<T> Clone for Rc<T> {
	fn clone(&self) -> Result<Self, Error> {
		match self.inner.clone() {
			Ok(mut inner) => {
				let rci = inner.as_mut();
				aadd!(&mut rci.count, 1);
				Ok(Self { inner })
			}
			Err(e) => Err(e),
		}
	}
}

use core::mem::{drop, replace, zeroed};
impl<T> Drop for Rc<T> {
	fn drop(&mut self) {
		let rci = self.inner.as_mut();
		let count = asub!(&mut rci.count, 1);
		if count == 1 {
			let value = replace(&mut rci.value, unsafe { zeroed() });
			drop(value);
		}

		if count == 1 {
			unsafe {
				self.inner.unleak();
			}
		}
	}
}

impl<T> Deref for RcInner<T> {
	type Target = T;
	fn deref(&self) -> &Self::Target {
		&self.value
	}
}

impl<T> DerefMut for RcInner<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.value
	}
}

impl<T> Rc<T> {
	pub fn new(value: T) -> Result<Self, Error> {
		match Box::new(RcInner { value, count: 1 }) {
			Ok(mut inner) => {
				unsafe {
					inner.leak();
				}
				Ok(Self { inner })
			}
			Err(e) => Err(e),
		}
	}

	pub fn get(&self) -> &T {
		&self.inner.value
	}

	pub fn get_mut(&mut self) -> Option<&mut T> {
		if aload!(&mut (*self.inner).count) == 1 {
			Some(&mut self.inner.value)
		} else {
			None
		}
	}

	pub unsafe fn get_mut_unchecked(&mut self) -> &mut T {
		&mut self.inner.value
	}
}

#[cfg(test)]
mod test {
	#![allow(static_mut_refs)]
	use super::*;
	use std::string::String;

	#[test]
	fn test_rc1() {
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

	static mut VTEST: usize = 0;

	struct MyType {
		v: usize,
	}

	impl Drop for MyType {
		fn drop(&mut self) {
			unsafe {
				VTEST += 1;
			}
		}
	}

	#[test]
	fn test_rc2() {
		{
			let x = Rc::new(MyType { v: 1 }).unwrap();
			assert_eq!(x.get().v, 1);
			let _y = x.clone();
			let _z = MyType { v: 2 };
		}
		unsafe {
			assert_eq!(VTEST, 2);
		}
	}
}
