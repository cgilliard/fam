use core::marker::Sized;
use core::mem::size_of;
use core::ptr::{drop_in_place, null_mut, write};
use prelude::*;
use sys::{alloc, release};

pub struct Box<T: ?Sized> {
	ptr: Pointer<T>,
}

impl<T: ?Sized> Drop for Box<T> {
	fn drop(&mut self) {
		if !self.ptr.get_bit() {
			let value_ptr = self.ptr.raw();
			unsafe {
				drop_in_place(value_ptr);
				if !value_ptr.is_null() {
					release(value_ptr as *mut u8);
				}
			}
		}
	}
}
/*
 *                 if !self.leak {
						let value_ptr: *mut T = self.as_mut_ptr();
						unsafe {
								drop_in_place(value_ptr);
								if !self.ptr.is_null() {
										release(self.ptr as *mut u8);
								}
						}
				}
*/

impl<T> Box<T> {
	pub fn new(t: T) -> Result<Self, Error> {
		let size = size_of::<T>();
		let ptr = if size == 0 {
			let mut ptr = Pointer::new(null_mut());
			ptr.set_bit(true);
			ptr
		} else {
			let mut ptr = unsafe {
				let rptr = alloc(size) as *mut T;
				if rptr.is_null() {
					return Err(ErrorKind::Alloc.into());
				}
				write(rptr, t);
				Pointer::new(rptr)
			};
			ptr.set_bit(false);
			ptr
		};
		Ok(Box { ptr })
	}
}

impl<T: ?Sized> Box<T> {
	pub fn leak(&mut self) {
		self.ptr.set_bit(true);
	}

	pub fn unleak(&mut self) {
		self.ptr.set_bit(false);
	}

	pub fn from_raw(ptr: *mut T) -> Box<T> {
		if ptr.is_null() {
			let mut ptr = Pointer::new(ptr);
			ptr.set_bit(true);
			Box { ptr }
		} else {
			Box {
				ptr: Pointer::new(ptr),
			}
		}
	}

	/*
	pub fn as_ref(&self) -> &T {
		unsafe { &*self.ptr.raw() }
	}
		*/

	pub fn as_mut(&mut self) -> &mut T {
		unsafe { &mut *self.ptr.raw() }
	}
	/*
		pub fn as_ptr(&self) -> *const T {
			self.ptr.raw()
		}
	*/

	pub fn as_mut_ptr(&mut self) -> *mut T {
		self.ptr.raw()
	}
}
