use core::clone::Clone as CoreClone;
use core::marker::{Copy, Sized, Unsize};
use core::ops::CoerceUnsized;
use prelude::*;
use sys::{ptr_add, resize};

pub struct Pointer<T: ?Sized> {
	ptr: *mut T,
}

impl<T: ?Sized> CoreClone for Pointer<T> {
	fn clone(&self) -> Self {
		*self
	}
}

impl<T: ?Sized> Copy for Pointer<T> {}

impl<T, U> CoerceUnsized<Pointer<U>> for Pointer<T>
where
	T: Unsize<U> + ?Sized,
	U: ?Sized,
{
}

impl<T: ?Sized> Pointer<T> {
	pub fn new(ptr: *mut T) -> Self {
		Self { ptr }
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

	pub fn resize<R>(&mut self, n: usize) -> Result<Pointer<R>, Error> {
		let ptr = unsafe { resize(self.ptr as *mut u8, n) };
		if ptr.is_null() {
			Err(ErrorKind::Alloc.into())
		} else {
			Ok(Pointer { ptr: ptr as *mut R })
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

impl<T> Pointer<T> {
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
		ptr: Pointer<T>,
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
				let ptr = Pointer::new(ptr as *mut T);
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
	}
}
