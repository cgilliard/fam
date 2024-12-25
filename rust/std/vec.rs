use core::cmp::PartialEq;
use core::iter::{IntoIterator, Iterator};
use core::marker::PhantomData;
use core::mem::{replace, size_of, zeroed};
use core::ops::{Drop, Index, IndexMut, Range};
use core::ptr::{copy_nonoverlapping, null_mut};
use core::slice::{from_raw_parts, from_raw_parts_mut};
use prelude::*;
use sys::{alloc, release, resize, write};

pub struct Vec<T> {
	value: Pointer<u8>,
	capacity: usize,
	elements: usize,
	_marker: PhantomData<T>,
}

impl<T: PartialEq> PartialEq for Vec<T> {
	fn eq(&self, other: &Vec<T>) -> bool {
		if self.len() != other.len() {
			false
		} else {
			for i in 0..self.len() {
				if self[i] != other[i] {
					return false;
				}
			}
			true
		}
	}
}

#[macro_export]
macro_rules! vec {
		($($elem:expr),*) => {
                    #[allow(unused_mut)]
                    {
				let mut vec = Vec::new();
				let mut err: Error = ErrorKind::Unknown.into();
				$(
					if err.kind == ErrorKind::Unknown {
						match vec.push($elem) {
							Ok(_) => {},
							Err(e) => err = e,
						}
					}
				)*
				if err.kind != ErrorKind::Unknown {
					Err(err)
				} else {
					Ok(vec)
				}
		    }
                };
}

pub struct VecIterator<T> {
	vec: Vec<T>,
	index: usize,
}

use core::option::Option as CoreOption;

impl<T> Iterator for VecIterator<T> {
	type Item = T;

	fn next(&mut self) -> CoreOption<Self::Item> {
		let size = size_of::<T>();
		if self.index < self.vec.elements {
			let ptr = self.vec.value.raw() as *const u8;
			let ptr = unsafe { ptr.add(self.index * size) as *mut T };
			let element = unsafe { replace(&mut *ptr, zeroed()) };
			self.index += 1;
			CoreOption::Some(element)
		} else {
			self.vec.elements = 0;
			CoreOption::None
		}
	}
}

impl<T> IntoIterator for Vec<T> {
	type Item = T;
	type IntoIter = VecIterator<T>;

	fn into_iter(self) -> Self::IntoIter {
		let ret = VecIterator {
			vec: self,
			index: 0,
		};
		ret
	}
}

use core::ptr;
impl<T> Drop for Vec<T> {
	fn drop(&mut self) {
		for i in 0..self.elements {
			unsafe {
				let ptr = (self.value.raw() as *const u8).add(i * size_of::<T>()) as *mut T;
				ptr::drop_in_place(ptr);
			}
		}
		let raw = self.value.raw();
		if !raw.is_null() {
			unsafe {
				release(raw);
			}
		}
	}
}

impl<T> Index<usize> for Vec<T> {
	type Output = T;

	fn index(&self, index: usize) -> &Self::Output {
		if index >= self.elements as usize {
			panic!("array index out of bounds!");
		}

		unsafe {
			let target = self.value.raw() as *const T;
			let target = target.add(index);
			&*(target as *const T)
		}
	}
}

impl<T> IndexMut<usize> for Vec<T> {
	fn index_mut(&mut self, index: usize) -> &mut Self::Output {
		if index >= self.elements as usize {
			panic!("array index out of bounds!");
		}

		unsafe {
			let target = self.value.raw() as *const T;
			let target = target.add(index);
			&mut *(target as *mut T)
		}
	}
}

impl<T> Index<Range<usize>> for Vec<T> {
	type Output = [T];

	fn index(&self, range: Range<usize>) -> &Self::Output {
		if range.start > range.end || range.end > self.elements {
			panic!("Index out of bounds");
		}

		let element_size = size_of::<T>();
		let start_offset = range.start * element_size;

		unsafe {
			let ptr = (self.value.raw() as *mut T).add(start_offset) as *const T;
			from_raw_parts(ptr, range.end - range.start)
		}
	}
}

impl<T> IndexMut<Range<usize>> for Vec<T> {
	fn index_mut(&mut self, range: Range<usize>) -> &mut Self::Output {
		if range.start > range.end || range.end > self.elements {
			panic!("Index out of bounds");
		}

		let element_size = size_of::<T>();
		let start_offset = range.start * element_size;

		unsafe {
			let ptr = (self.value.raw() as *mut T).add(start_offset) as *mut T;
			from_raw_parts_mut(ptr, range.end - range.start)
		}
	}
}

impl<T> Vec<T> {
	pub fn new() -> Self {
		Self {
			value: Pointer::new(null_mut()),
			capacity: 0,
			elements: 0,
			_marker: PhantomData,
		}
	}

	pub fn push(&mut self, v: T) -> Result<(), Error> {
		let size = size_of::<T>();

		if self.elements + 1 > self.capacity {
			if !self.resize_impl(self.elements + 1) {
				return Err(ErrorKind::Alloc.into());
			}
		}

		let dest_ptr = self.value.raw() as *mut u8;
		unsafe {
			let dest_ptr = dest_ptr.add(size * self.elements) as *mut T;
			ptr::write(dest_ptr, v);
		}
		self.elements += 1;

		Ok(())
	}

	fn next_power_of_two(mut n: usize) -> usize {
		if n == 0 {
			return 0;
		}
		n -= 1;
		n |= n >> 1;
		n |= n >> 2;
		n |= n >> 4;
		n |= n >> 8;
		n |= n >> 16;
		n |= n >> 32;
		n + 1
	}

	fn resize_impl(&mut self, needed: usize) -> bool {
		if needed == 0 {
			if !self.value.raw().is_null() {
				unsafe {
					release(self.value.raw());
				}
			}
			self.value = Pointer::new(null_mut());
		}
		let ncapacity = Self::next_power_of_two(needed);

		let rptr = self.value.raw();

		let nptr = if self.capacity == 0 {
			unsafe { alloc(ncapacity * size_of::<T>()) }
		} else {
			unsafe { crate::sys::resize(rptr as *mut u8, ncapacity * size_of::<T>()) }
		};
		if !nptr.is_null() {
			self.capacity = ncapacity;
			let nptr = Pointer::new(nptr as *mut u8);
			if self.value.raw().is_null() {
				self.value = nptr;
			} else {
				self.value = nptr;
			}
			true
		} else {
			false
		}
	}

	pub fn len(&self) -> usize {
		self.elements
	}

	pub fn clear(&mut self) {
		self.resize_impl(0);
		self.elements = 0;
	}

	pub fn as_mut_ptr(&mut self) -> *mut u8 {
		null_mut()
	}

	pub fn resize(&mut self, n: usize) -> Result<(), Error> {
		if self.resize_impl(n) {
			self.elements = n;
			Ok(())
		} else {
			Err(ErrorKind::Alloc.into())
		}
	}

	pub fn append(&mut self, v: &Vec<T>) -> Result<(), Error> {
		Ok(())
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use core::ops::Drop;
	use sys::getalloccount;

	#[test]
	fn test_vec1() {
		let mut x = vec![1, 2, 3, 4, 5, 6].unwrap();
		assert_eq!(x[0], 1);
		assert_eq!(x[1], 2);
		assert_eq!(x[2], 3);
		assert_eq!(x[3], 4);
		assert_eq!(x[4], 5);
		assert_eq!(x[5], 6);
		assert_eq!(x.len(), 6);
		x[5] += 1;
		assert_eq!(x[5], 7);
	}

	#[test]
	fn test_vec2() {
		let initial = unsafe { getalloccount() };
		{
			let mut v1 = Vec::new();
			for i in 0..100000 {
				assert!(v1.push(i).is_ok());
				assert_eq!(v1[i], i);
			}

			for i in 0..100000 {
				v1[i] = i + 100;
			}
			for i in 0..100000 {
				assert_eq!(v1[i], i + 100);
			}

			let v2 = vec![1, 2, 3].unwrap();
			let mut count = 0;
			for x in v2 {
				count += 1;
				assert_eq!(x, count);
			}
			assert_eq!(count, 3);
		}
		assert_eq!(initial, unsafe { getalloccount() });
	}

	use core::fmt::Debug;
	use core::fmt::Error as CoreError;
	use core::fmt::Formatter;
	use core::result::Result as CoreResult;

	impl<T> Debug for Vec<T> {
		fn fmt(&self, _: &mut Formatter<'_>) -> CoreResult<(), CoreError> {
			CoreResult::Ok(())
		}
	}

	/*
	#[test]
	fn test_vec_append() {
		let initial = unsafe { getalloccount() };
		{
			let mut v1 = vec![1, 2, 3].unwrap();
			let v2 = vec![4, 5, 6].unwrap();
			assert!(v1.append(&v2).is_ok());

			assert_eq!(v1, vec![1, 2, 3, 4, 5, 6].unwrap());
			assert!(v1 != vec![1, 2, 3, 4, 6, 6].unwrap());
			assert!(v1 == vec![1, 2, 3, 4, 5, 6].unwrap());
		}
		assert_eq!(initial, unsafe { getalloccount() });
	}
		*/

	struct DropTest {
		x: u32,
	}

	static mut VTEST: u32 = 0;

	impl Drop for DropTest {
		fn drop(&mut self) {
			unsafe {
				VTEST += 1;
			}
		}
	}

	#[test]
	fn test_vec_drop() {
		let x = DropTest { x: 8 };

		let initial = unsafe { getalloccount() };
		{
			let mut v: Vec<DropTest> = vec![].unwrap();
			assert!(v.resize(1).is_ok());
			v[0] = x;
			assert_eq!(v[0].x, 8);
		}

		assert_eq!(unsafe { VTEST }, 2);
		assert_eq!(initial, unsafe { getalloccount() });
	}

	#[test]
	fn test_vec_iter_drop() {
		let initial = unsafe { getalloccount() };
		{
			unsafe {
				VTEST = 0;
			}
			{
				let v = vec![DropTest { x: 1 }, DropTest { x: 2 }, DropTest { x: 3 }].unwrap();
				for y in v {
					let _z = y;
				}
			}
			assert_eq!(unsafe { VTEST }, 3);
		}
		assert_eq!(initial, unsafe { getalloccount() });
	}
}
