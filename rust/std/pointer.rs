use core::clone::Clone as CoreClone;
use core::cmp::PartialEq;
use core::marker::{Copy, Sized, Unsize};
use core::mem::size_of;
use core::ops::{CoerceUnsized, Deref, DerefMut};
use core::ptr::{null_mut, write};
use core::str::from_utf8_unchecked;
use prelude::*;
use std::util::u128_to_str;
use sys::{ptr_add, resize};

pub struct Ptr<T: ?Sized> {
	ptr: *mut T,
}

impl<T: ?Sized> PartialEq for Ptr<T> {
	fn eq(&self, other: &Self) -> bool {
		self.raw() as *mut u8 as usize == other.raw() as *mut u8 as usize
	}
}

impl<T: ?Sized> Display for Ptr<T> {
	fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
		let v = self.raw() as *mut u8 as u128;
		let mut buf = [0u8; 64];
		buf[0] = b'0';
		buf[1] = b'x';
		let len = u128_to_str(v, 2, &mut buf, 16);
		unsafe { f.write_str(from_utf8_unchecked(&buf), len + 2) }
	}
}

impl<T: ?Sized> CoreClone for Ptr<T> {
	fn clone(&self) -> Self {
		*self
	}
}

impl<T: ?Sized> Copy for Ptr<T> {}

impl<T, U> CoerceUnsized<Ptr<U>> for Ptr<T>
where
	T: Unsize<U> + ?Sized,
	U: ?Sized,
{
}

impl<T: ?Sized> Deref for Ptr<T> {
	type Target = T;
	fn deref(&self) -> &Self::Target {
		unsafe { &*self.raw() }
	}
}

impl<T> DerefMut for Ptr<T>
where
	T: ?Sized,
{
	fn deref_mut(&mut self) -> &mut Self::Target {
		unsafe { &mut *self.raw() }
	}
}

impl<T> Ptr<T> {
	pub fn alloc(t: T) -> Result<Self, Error> {
		let ptr = unsafe { crate::sys::alloc(size_of::<T>()) } as *mut T;

		if ptr.is_null() {
			Err(ErrorKind::Alloc.into())
		} else {
			unsafe {
				write(ptr, t);
			}
			Ok(Self { ptr })
		}
	}

	pub fn null() -> Self {
		let ptr = null_mut();
		Self { ptr }
	}
}

impl<T: ?Sized> Ptr<T> {
	pub fn new(ptr: *mut T) -> Self {
		Self { ptr }
	}

	pub fn is_null(&self) -> bool {
		self.raw().is_null()
	}

	pub fn set_bit(&mut self, v: bool) {
		let ptr = (&mut self.ptr) as *mut _ as *mut *mut u8;
		unsafe {
			if v && (self.ptr as *mut u8 as usize) % 2 == 0 {
				ptr_add(ptr as *mut _, 1); // Add 1 to set the bit
			} else if !v && (self.ptr as *mut u8 as usize) % 2 != 0 {
				ptr_add(ptr as *mut _, -1); // Subtract 1 to clear the bit
			}
		}
	}

	pub fn get_bit(&self) -> bool {
		self.ptr as *mut u8 as usize % 2 != 0
	}

	pub fn raw(&self) -> *mut T {
		if self.get_bit() {
			let mut ret = self.ptr;
			unsafe {
				ptr_add(&mut ret as *mut _ as *mut u8, -1);
			}
			ret
		} else {
			self.ptr
		}
	}

	pub fn release(&self) {
		unsafe {
			crate::sys::release(self.raw() as *mut u8);
		}
	}

	pub fn resize<R>(&mut self, n: usize) -> Result<Ptr<R>, Error> {
		let ptr = unsafe { resize(self.ptr as *mut u8, n) };
		if ptr.is_null() {
			Err(ErrorKind::Alloc.into())
		} else {
			Ok(Ptr { ptr: ptr as *mut R })
		}
	}

	pub fn byte_add(&self, n: i64) -> *mut u8 {
		let mut ret = self.raw() as *mut u8;
		unsafe {
			ptr_add(&mut ret as *mut _ as *mut u8, n);
		}
		ret
	}

	pub fn as_ref(&self) -> &T {
		unsafe { &(*self.raw()) }
	}

	pub fn as_mut(&mut self) -> &mut T {
		unsafe { &mut (*self.raw()) }
	}
}

impl<T> Ptr<T> {
	pub fn offt(&mut self, n: usize) -> *mut T {
		unsafe { (self.raw() as *mut u8).add(n) as *mut T }
	}

	pub fn add(&mut self, n: usize) -> *mut T {
		unsafe { (self.raw() as *mut T).add(n) }
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use core::mem::size_of;
	use core::ptr::write;
	use sys::{alloc, release};

	#[derive(Clone)]
	struct MyBox<T: ?Sized> {
		ptr: Ptr<T>,
	}

	impl<T: ?Sized> Drop for MyBox<T> {
		fn drop(&mut self) {
			unsafe {
				release(self.ptr.raw() as *mut u8);
			}
		}
	}

	impl<T> MyBox<T> {
		fn new(t: T) -> Self {
			unsafe {
				let ptr = alloc(size_of::<T>());
				write(ptr as *mut T, t);
				let ptr = Ptr::new(ptr as *mut T);
				Self { ptr }
			}
		}

		fn as_ref(&mut self) -> &T {
			unsafe { &*(self.ptr.raw() as *mut T) }
		}

		fn get_bit(&self) -> bool {
			self.ptr.get_bit()
		}
		fn set_bit(&mut self, v: bool) {
			self.ptr.set_bit(v);
		}
	}

	#[test]
	fn test_pointer() {
		let mut b = MyBox::new(123);
		b.set_bit(false);
		assert!(!b.get_bit());
		assert_eq!(b.as_ref(), &123);

		let mut b2 = MyBox::new(456);
		b2.set_bit(true);
		assert!(b2.get_bit());
		assert_eq!(b2.as_ref(), &456);

		let ptr = Ptr::alloc(1usize).unwrap();
		let ptr2 = Ptr::new(ptr.raw());
		let ptr3 = Ptr::alloc(2usize).unwrap();
		let ptr4 = Ptr::alloc(2usize).unwrap();

		assert!(ptr == ptr2);
		assert!(ptr != ptr3);
		assert!(ptr != ptr4);
		assert!(ptr3 != ptr4);
		ptr.release();
		ptr3.release();
		ptr4.release();
	}
}
