use core::marker::{Sized, Unsize};
use core::mem::size_of;
use core::ops::{CoerceUnsized, Deref, DerefMut};
use core::ptr;
use core::ptr::{drop_in_place, null_mut};
use core::slice::from_raw_parts_mut;
use prelude::*;
use sys::{alloc, release};

#[derive(PartialEq, Debug)]
pub struct Box<T: ?Sized> {
	ptr: *mut T,
	leak: bool,
}

impl<T: ?Sized> Drop for Box<T> {
	fn drop(&mut self) {
		if !self.leak {
			let value_ptr: *mut T = self.as_mut_ptr();
			unsafe {
				drop_in_place(value_ptr);
				if !self.ptr.is_null() {
					release(self.ptr as *mut u8);
				}
			}
		}
	}
}

impl<T: Clone> Clone for Box<T> {
	fn clone(&self) -> Result<Self, Error> {
		match self.as_ref().clone() {
			Ok(value) => Box::new(value),
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
		unsafe { &*self.ptr }
	}
}

impl<T> DerefMut for Box<T>
where
	T: ?Sized,
{
	fn deref_mut(&mut self) -> &mut Self::Target {
		unsafe { &mut *self.ptr }
	}
}

impl<T> Box<T>
where
	T: ?Sized,
{
	pub unsafe fn from_raw(ptr: *mut T) -> Box<T> {
		Box { ptr, leak: false }
	}

	pub unsafe fn unleak(&mut self) {
		self.leak = false;
	}

	pub unsafe fn leak(&mut self) {
		self.leak = true;
	}

	pub fn as_ref(&self) -> &T {
		unsafe { &*self.ptr }
	}

	pub fn as_mut(&mut self) -> &mut T {
		unsafe { &mut *self.ptr }
	}

	pub fn as_ptr(&self) -> *const T {
		self.ptr
	}

	pub fn as_mut_ptr(&mut self) -> *mut T {
		self.ptr
	}
	pub unsafe fn into_inner(self) -> *mut T {
		let value = self.ptr;
		value
	}
}

impl<T> Box<T> {
	pub fn new(t: T) -> Result<Self, Error> {
		let size = size_of::<T>();
		let ptr;
		let leak;
		if size == 0 {
			ptr = null_mut();
			leak = true;
		} else {
			leak = false;
			unsafe {
				ptr = alloc(size) as *mut T;
				if ptr.is_null() {
					return Err(ErrorKind::Alloc.into());
				}
				ptr::write(ptr, t);
			}
		}

		Ok(Box { ptr, leak })
	}
}

impl Box<[u8]> {
	pub fn new_zeroed_byte_slice(len: usize) -> Result<Box<[u8]>, Error> {
		let ptr;
		unsafe {
			ptr = alloc(len);
			ptr::write_bytes(ptr, 0, len);
		}
		if ptr.is_null() {
			return Err(ErrorKind::Alloc.into());
		}
		let slice = unsafe { Box::from_raw(from_raw_parts_mut(ptr, len)) };

		Ok(slice)
	}
}

impl<T, U> CoerceUnsized<Box<U>> for Box<T>
where
	T: Unsize<U> + ?Sized,
	U: ?Sized,
{
}

#[cfg(test)]
mod test {
	use super::*;
	use core::ops::Fn;
	use sys::getalloccount;

	#[test]
	fn test_box1() {
		let initial = unsafe { getalloccount() };
		{
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
		assert_eq!(initial, unsafe { getalloccount() });
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
	fn test_box2() {
		let initial = unsafe { getalloccount() };
		{
			let mut b1: Box<TestSample> = Box::new(TestSample { data: 1 }).unwrap();
			unsafe {
				b1.leak();
			}
			let b2: Box<dyn GetData> = unsafe { Box::from_raw(b1.as_mut_ptr()) };
			assert_eq!(b2.get_data(), 1);

			let b3: Box<dyn GetData> = Box::new(TestSample { data: 2 }).unwrap();
			assert_eq!(b3.get_data(), 2);

			let b4 = Box::new(|x| 5 + x).unwrap();
			assert_eq!(b4(5), 10);
		}

		assert_eq!(initial, unsafe { getalloccount() });
	}

	struct BoxTest<CLOSURE>
	where
		CLOSURE: Fn(i32) -> i32,
	{
		x: Box<dyn GetData>,
		y: Box<CLOSURE>,
		z: Box<[u8]>,
	}

	struct BoxTest2<T> {
		v: Box<[T]>,
	}

	#[test]
	fn test_box3() {
		let initial = unsafe { getalloccount() };
		{
			let x = BoxTest {
				x: Box::new(TestSample { data: 8 }).unwrap(),
				y: Box::new(|x| x + 4).unwrap(),
				z: Box::new([3u8; 32]).unwrap(),
			};

			assert_eq!(x.x.get_data(), 8);
			assert_eq!((x.y)(14), 18);
			assert_eq!(x.z[5], 3u8);

			let y = BoxTest2 {
				v: Box::new([5u64; 40]).unwrap(),
			};

			assert_eq!(y.v[9], 5);
		}
		assert_eq!(initial, unsafe { getalloccount() });
	}

	#[test]
	fn test_box4() {
		let initial = unsafe { getalloccount() };
		{
			let mut box1 = Box::new([9u8; 992]).unwrap();
			for i in 0..992 {
				assert_eq!(9u8, box1.as_ref()[i]);
			}
			let box1_mut = box1.as_mut();
			for i in 0..992 {
				box1_mut[i] = 8;
			}
			for i in 0..992 {
				assert_eq!(8u8, box1.as_ref()[i]);
			}

			let mut box2 = Box::new_zeroed_byte_slice(20000).unwrap();
			for i in 0..20000 {
				box2.as_mut()[i] = 10;
			}

			for i in 0..20000 {
				assert_eq!(box2.as_ref()[i], 10);
			}
		}
		assert_eq!(initial, unsafe { getalloccount() });
	}

	static mut COUNT: i32 = 0;

	struct DropBox {
		x: u32,
	}

	impl Drop for DropBox {
		fn drop(&mut self) {
			assert_eq!(self.x, 1);
			unsafe {
				COUNT += 1;
			}
		}
	}

	#[test]
	fn test_drop_box() {
		let initial = unsafe { getalloccount() };
		{
			let _big = Box::new_zeroed_byte_slice(100000);
			let _v = Box::new(DropBox { x: 1 }).unwrap();
			assert_eq!(unsafe { COUNT }, 0);
		}
		assert_eq!(unsafe { COUNT }, 1);

		assert_eq!(initial, unsafe { getalloccount() });
	}

	static mut CLONE_DROP_COUNT: i32 = 0;

	#[derive(Debug, PartialEq)]
	struct CloneBox {
		x: u32,
	}

	impl Clone for CloneBox {
		fn clone(&self) -> Result<Self, Error> {
			Ok(Self { x: self.x })
		}
	}

	impl Drop for CloneBox {
		fn drop(&mut self) {
			assert_eq!(self.x, 10);
			unsafe {
				CLONE_DROP_COUNT += 1;
			}
		}
	}

	#[test]
	fn test_clone_box() {
		{
			let x = CloneBox { x: 10 };
			let y = Box::new(x).unwrap();
			let z = Box::clone(&y).unwrap();
			assert_eq!(*z, *y);
			assert_eq!(unsafe { CLONE_DROP_COUNT }, 0);
		}
		assert_eq!(unsafe { CLONE_DROP_COUNT }, 2);
	}
}
