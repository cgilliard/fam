use crate::*;
use core::cmp::PartialEq;
use core::iter::{IntoIterator, Iterator};
use core::marker::PhantomData;
use core::mem::{replace, size_of, zeroed};
use core::ops::{Index, IndexMut, Range};
use core::ptr::copy_nonoverlapping;
use core::slice::from_raw_parts;

// TODO: PartialEq should be implemented differently item by item comparison
#[derive(Debug)]
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
		($($elem:expr),*) => {{
				let mut vec = Vec::new();
				let mut err = err!(NoError);
				$(
					if err.kind == NoError {
						match vec.push($elem) {
							Ok(_) => {},
							Err(e) => err = e,
						}
					}
				)*
				if(err.kind != NoError) {
					Err(err)
				} else {
					Ok(vec)
				}
		}};
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
			CoreOption::None
		}
	}
}

impl<T> IntoIterator for Vec<T> {
	type Item = T;
	type IntoIter = VecIterator<T>;

	fn into_iter(self) -> Self::IntoIter {
		VecIterator {
			vec: self,
			index: 0,
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

		let element_size = core::mem::size_of::<T>();
		let start_offset = range.start * element_size;

		unsafe {
			let ptr = (self.value.as_ptr() as *mut T).add(start_offset) as *const T;
			from_raw_parts(ptr, range.end - range.start)
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
				return Err(err!(Alloc));
			}
		}
		let elem = self.elements;
		self.elements += 1;
		self[elem] = v;
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
		// use slab sizes for resize boundaries
		if needed > 4064 {
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

	pub fn resize(&mut self, n: usize) -> Result<(), Error> {
		let size = size_of::<T>();
		let needed = size * n;
		if !self.resize_impl(needed) {
			Err(err!(Alloc))
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
				return Err(err!(Alloc));
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
	#[test]
	fn test_vec() {
		let mut v1 = Vec::new();
		for i in 0..100000 {
			v1.push(i);
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

	#[test]
	fn test_vec_append() {
		let mut v1 = vec![1, 2, 3].unwrap();
		let v2 = vec![4, 5, 6].unwrap();
		v1.append(&v2);

		assert_eq!(v1, vec![1, 2, 3, 4, 5, 6].unwrap());
		assert!(v1 != vec![1, 2, 3, 4, 6, 6].unwrap());
		assert!(v1 == vec![1, 2, 3, 4, 5, 6].unwrap());
	}
}
