use core::cmp::PartialEq;
use core::iter::{IntoIterator, Iterator};
use core::marker::PhantomData;
use core::mem::{replace, size_of, zeroed};
use core::ops::{Drop, Index, IndexMut, Range};
use core::ptr::copy_nonoverlapping;
use core::slice::{from_raw_parts, from_raw_parts_mut};
use prelude::*;

pub struct Vec<T> {
	value: Box<[u8]>,
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
			let ptr = self.vec.value.as_ptr() as *const u8;
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
				let ptr = (self.value.as_ptr() as *const u8).add(i * size_of::<T>()) as *mut T;
				ptr::drop_in_place(ptr);
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
			let target = self.value.as_ptr() as *const T;
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
			let target = self.value.as_mut_ptr() as *mut T;
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
			let ptr = (self.value.as_ptr() as *mut T).add(start_offset) as *const T;
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
			let ptr = (self.value.as_ptr() as *mut T).add(start_offset) as *mut T;
			from_raw_parts_mut(ptr, range.end - range.start)
		}
	}
}

impl<T> Vec<T> {
	pub fn new() -> Self {
		Self {
			value: Box::new([]).unwrap(),
			capacity: 0,
			elements: 0,
			_marker: PhantomData,
		}
	}

	pub fn push(&mut self, v: T) -> Result<(), Error> {
		let size = size_of::<T>();
		let needed = size * (self.elements + 1);
		if needed > self.capacity {
			if !self.resize_impl(needed) {
				return Err(ErrorKind::Alloc.into());
			}
		}
		let dest_ptr = self.value.as_mut_ptr() as *mut u8;
		unsafe {
			let dest_ptr = dest_ptr.add(size * self.elements) as *mut T;
			ptr::write(dest_ptr, v);
		}
		self.elements += 1;
		Ok(())
	}

	fn copy_box(&mut self, needed: usize, mut newbox: Box<[u8]>) {
		let copy_size = if self.capacity > needed {
			needed
		} else {
			self.capacity
		};
		if copy_size > 0 {
			unsafe {
				copy_nonoverlapping(
					self.value.as_ptr() as *const u8,
					newbox.as_mut_ptr() as *mut u8,
					copy_size,
				);
			}
		}
		self.value = newbox;
	}

	fn resize_impl(&mut self, needed: usize) -> bool {
		if needed == 0 {
			// unwrap ok because zero sized box no alloc
			self.value = Box::new([]).unwrap();
			self.capacity = 0;
			self.elements = 0;
		} else if needed > 4064 {
			let npages = 1 + pages!(needed);
			match Box::new_zeroed_byte_slice(npages * page_size!()) {
				Ok(newbox) => self.copy_box(needed, newbox),
				Err(_) => return false,
			}
			self.capacity = npages * page_size!();
		} else if needed > 2016 {
			match Box::new([0u8; 4064]) {
				Ok(newbox) => self.copy_box(needed, newbox),
				Err(_) => return false,
			}
			self.capacity = 4064;
		} else if needed > 992 {
			match Box::new([0u8; 2016]) {
				Ok(newbox) => self.copy_box(needed, newbox),
				Err(_) => return false,
			}
			self.capacity = 2016;
		} else if needed > 480 {
			match Box::new([0u8; 992]) {
				Ok(newbox) => self.copy_box(needed, newbox),
				Err(_) => return false,
			}
			self.capacity = 992;
		} else if needed > 224 {
			match Box::new([0u8; 480]) {
				Ok(newbox) => self.copy_box(needed, newbox),
				Err(_) => return false,
			}
			self.capacity = 480;
		} else if needed > 96 {
			match Box::new([0u8; 224]) {
				Ok(newbox) => self.copy_box(needed, newbox),
				Err(_) => return false,
			}
			self.capacity = 224;
		} else if needed > 32 {
			match Box::new([0u8; 96]) {
				Ok(newbox) => self.copy_box(needed, newbox),
				Err(_) => return false,
			}
			self.capacity = 96;
		} else {
			match Box::new([0u8; 32]) {
				Ok(newbox) => self.copy_box(needed, newbox),
				Err(_) => return false,
			}
			self.capacity = 32;
		};
		return true;
	}

	pub fn len(&self) -> usize {
		self.elements
	}

	pub fn clear(&mut self) {
		self.elements = 0;
	}

	pub fn as_mut_ptr(&mut self) -> *mut u8 {
		self.value.as_mut_ptr() as *mut u8
	}

	pub fn resize(&mut self, n: usize) -> Result<(), Error> {
		let size = size_of::<T>();
		let needed = size * n;
		if !self.resize_impl(needed) {
			Err(ErrorKind::Alloc.into())
		} else {
			self.elements = n;
			Ok(())
		}
	}

	pub fn append(&mut self, v: &Vec<T>) -> Result<(), Error> {
		let size = size_of::<T>();
		let len = v.len();
		let needed = size * (self.elements + len);
		if needed > self.capacity {
			if !self.resize_impl(needed) {
				return Err(ErrorKind::Alloc.into());
			}
		}

		let dest_ptr = self.value.as_mut_ptr() as *mut u8;
		unsafe {
			let dest_ptr = dest_ptr.add(size * len) as *mut u8;
			copy_nonoverlapping(v.value.as_ptr() as *mut u8, dest_ptr, size * len);
		}

		self.elements += len;
		Ok(())
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use core::ops::Drop;
	use sys::getalloccount;

	#[test]
	fn test_vec() {
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
			todo!()
		}
	}

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
