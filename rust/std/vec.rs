use core::iter::Iterator;
use core::marker::PhantomData;
use core::mem::size_of;
use core::ops::{Drop, Index, IndexMut};
use core::option::{Option, Option::None, Option::Some};
use core::ptr::{copy_nonoverlapping, null_mut};
use err;
use std::error::{Error, ErrorKind::*};
use std::result::{Result, Result::Err, Result::Ok};
use sys::{getpagesize, map, unmap};

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

pub struct Vec<T> {
	ptr: *mut u8,
	capacity: u64,
	elements: u64,
	_phantom_data: PhantomData<T>,
}

impl<T> Index<usize> for Vec<T> {
	type Output = T;

	fn index(&self, index: usize) -> &Self::Output {
		if index >= self.elements as usize {
			panic!("array index out of bounds!");
		}
		unsafe { &*((self.ptr.add(index * size_of::<T>())) as *const T) }
	}
}

impl<T> IndexMut<usize> for Vec<T> {
	fn index_mut(&mut self, index: usize) -> &mut Self::Output {
		if index >= self.elements as usize {
			panic!("array index out of bounds!");
		}
		unsafe { &mut *((self.ptr.add(index * size_of::<T>())) as *mut T) }
	}
}

pub struct VecIterator<'a, T> {
	vec: &'a Vec<T>,
	index: usize,
}

impl<'a, T> Iterator for VecIterator<'a, T> {
	type Item = &'a T;

	fn next(&mut self) -> Option<Self::Item> {
		if self.index < self.vec.elements as usize {
			let element = &self.vec[self.index]; // Use the Index trait
			self.index += 1;
			Some(element)
		} else {
			None
		}
	}
}

impl<T> Vec<T> {
	pub fn new() -> Self {
		Self {
			ptr: null_mut(),
			capacity: 0,
			elements: 0,
			_phantom_data: PhantomData,
		}
	}

	pub fn push(&mut self, v: T) -> Result<(), Error> {
		let size = size_of::<T>() as u64;
		let needed = size * (self.elements + 1);
		if needed > self.capacity * page_size!() {
			if !self.resize_impl(needed) {
				return Err(err!(Alloc));
			}
		}

		unsafe {
			let dest_ptr = self.ptr.add((self.elements * size) as usize);
			copy_nonoverlapping(&v as *const T as *const u8, dest_ptr, size as usize);
		}

		self.elements += 1;

		Ok(())
	}

	pub fn len(&self) -> u64 {
		self.elements
	}

	pub fn append(&mut self, v: &Vec<T>) -> Result<(), Error> {
		let size = size_of::<T>() as u64;
		let len = v.len();
		let needed = size * (self.elements + len);
		if needed > self.capacity * page_size!() {
			if !self.resize_impl(needed) {
				return Err(err!(Alloc));
			}
		}

		unsafe {
			let dest_ptr = self.ptr.add((self.elements * size) as usize);
			copy_nonoverlapping(v.ptr, dest_ptr, (size * len) as usize);
		}

		self.elements += len;
		Ok(())
	}

	pub fn resize(&mut self, n: u64) -> Result<(), Error> {
		let size = size_of::<T>() as u64;
		let needed = size * n;
		if !self.resize_impl(needed) {
			Err(err!(Alloc))
		} else {
			self.elements = n;
			Ok(())
		}
	}

	fn resize_impl(&mut self, needed: u64) -> bool {
		let pages = pages!(needed);
		let tmp = unsafe { map(pages) };
		if tmp.is_null() {
			false
		} else {
			let size = (pages * page_size!()) as usize;
			let cur = (self.capacity * page_size!()) as usize;

			unsafe {
				let n = if size > cur { cur } else { size };
				if n > 0 {
					copy_nonoverlapping(tmp, self.ptr, n);
				}
			}
			self.ptr = tmp;
			self.capacity = pages;
			true
		}
	}

	pub fn clear(&mut self) {
		self.elements = 0; // Reset the element count
	}

	pub fn iter(&self) -> VecIterator<T> {
		VecIterator {
			vec: self,
			index: 0,
		}
	}
}

impl<T> Drop for Vec<T> {
	fn drop(&mut self) {
		if self.capacity > 0 {
			unsafe {
				unmap(self.ptr, self.capacity);
			}
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;
	#[test]
	fn test_vec() {
		let mut x = Vec::new();
		let _ = x.push('a');
		let _ = x.push('b');
		assert_eq!(x[0], 'a');
		assert_eq!(x[1], 'b');
		x[0] = 'y';
		x[1] = 'z';
		assert_eq!(x[0], 'y');
		assert_eq!(x[1], 'z');
		let mut i = 0;
		for v in x.iter() {
			if i == 0 {
				assert_eq!(*v, 'y');
			} else {
				assert_eq!(*v, 'z');
			}
			i += 1;
		}
		assert_eq!(i, 2);

		let v: Result<Vec<i32>, Error> = vec![1, 2, 3];
		let vu = v.unwrap();
		assert_eq!(vu[0], 1);
		assert_eq!(vu[1], 2);
	}
}
