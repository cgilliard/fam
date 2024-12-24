use core::clone::Clone as CoreClone;
use core::marker::{Copy, PhantomData, Sized};

pub struct Pointer<T: ?Sized> {
	ptr: *mut u8,
	_marker: PhantomData<T>,
}

impl<T: ?Sized> CoreClone for Pointer<T> {
	fn clone(&self) -> Self {
		*self
	}
}

impl<T: ?Sized> Copy for Pointer<T> {}

impl<T: ?Sized> Pointer<T> {
	pub fn new(ptr: *mut T) -> Self {
		Self {
			ptr: ptr as *mut u8,
			_marker: PhantomData,
		}
	}
}

impl<T: ?Sized> Pointer<T> {
	pub fn byte_add(&self, n: usize) -> Pointer<T> {
		Pointer {
			ptr: unsafe { self.ptr.add(n) },
			_marker: PhantomData,
		}
	}
}

impl<T> Pointer<T> {
	pub fn offt(&mut self, n: usize) -> *mut T {
		unsafe { self.ptr.add(n) as *mut T }
	}

	pub fn add(&mut self, n: usize) -> *mut T {
		unsafe { (self.ptr as *mut T).add(n) }
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use core::mem::size_of;
	use core::ptr::write;
	use sys::{alloc, release};

	#[derive(Copy, Clone)]
	struct MyBox<T: ?Sized> {
		ptr: Pointer<T>,
	}

	impl<T> MyBox<T> {
		fn new(t: T) -> Self {
			unsafe {
				let ptr = alloc(8 + size_of::<T>());
				write(ptr.add(8) as *mut T, t);
				let ptr = Pointer::new(ptr as *mut T);
				Self { ptr }
			}
		}

		fn as_ref(&self) -> &T {
			unsafe { &*self.ptr.byte_add(8).add(0) }
		}
	}

	#[test]
	fn test_pointer() {
		let b = MyBox::new(123);
		assert_eq!(b.as_ref(), &123);
	}
}
