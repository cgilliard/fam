use core::iter::{IntoIterator, Iterator};
use core::marker::PhantomData;
use core::mem::{replace, size_of, zeroed};
use core::ops::Drop;
use core::ops::{Index, IndexMut};
use core::option::{Option, Option::None, Option::Some};
use core::ptr::{copy_nonoverlapping, null_mut};
use err;
use std::error::{Error, ErrorKind::Alloc};
use std::result::{Result, Result::Err, Result::Ok};
use sys::{map, unmap};

pub struct Vec<T> {
	ptr: *mut u8,
	capacity: usize,
	elements: usize,
	_phantom_data: PhantomData<T>,
	svo: [u8; 40],
}

#[macro_export]
macro_rules! vec {
        ($($elem:expr),*) => {{
                use std::vec::Vec;
                use std::result::Result::{Err,Ok};
                use std::error::{ErrorKind::NoError, Error};
                use err;
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

impl<T> Drop for Vec<T> {
	fn drop(&mut self) {
		if self.capacity > 0 {
			unsafe {
				unmap(self.ptr, self.capacity);
			}
		}
	}
}

pub struct VecIterator<T> {
	vec: Vec<T>,
	index: usize,
}

impl<T> Iterator for VecIterator<T> {
	type Item = T;

	fn next(&mut self) -> Option<Self::Item> {
		let size = size_of::<T>();
		if self.index < self.vec.elements {
			let element = if self.vec.capacity == 0 {
				let ptr = unsafe { self.vec.svo.as_ptr().add(self.index * size) as *mut T };
				unsafe { replace(&mut *ptr, zeroed()) }
			} else {
				let ptr = unsafe { self.vec.ptr.add(self.index * size) as *mut T };
				unsafe { replace(&mut *ptr, zeroed()) }
			};
			self.index += 1;
			Some(element)
		} else {
			None
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
			let target = if self.capacity != 0 {
				self.ptr.add(index * size_of::<T>())
			} else {
				self.svo.as_ptr().add(index * size_of::<T>())
			};
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
			let target = if self.capacity != 0 {
				self.ptr.add(index * size_of::<T>())
			} else {
				self.svo.as_mut_ptr().add(index * size_of::<T>())
			};
			&mut *(target as *mut T)
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
			svo: [0u8; 40],
		}
	}

	pub fn push(&mut self, v: T) -> Result<(), Error> {
		let size = size_of::<T>();
		let needed = size * (self.elements + 1);
		if needed < self.svo.len() {
			unsafe {
				let dest_ptr = self.svo.as_mut_ptr().add(self.elements * size);
				copy_nonoverlapping(&v as *const T as *const u8, dest_ptr, size);
			}
		} else {
			let copy_svo = self.capacity == 0 && self.elements != 0;
			if needed > self.capacity * page_size!() {
				if !self.resize_impl(needed) {
					return Err(err!(Alloc));
				}
			}
			if copy_svo {
				unsafe {
					copy_nonoverlapping(&self.svo as *const u8, self.ptr, self.elements * size);
				}
			}

			unsafe {
				let dest_ptr = self.ptr.add(self.elements * size);
				copy_nonoverlapping(&v as *const T as *const u8, dest_ptr, size);
			}
		}

		self.elements += 1;
		Ok(())
	}

	pub fn append(&mut self, v: &Vec<T>) -> Result<(), Error> {
		let size = size_of::<T>();
		let len = v.len();
		let needed = size * (self.elements + len);
		if needed < self.svo.len() {
			unsafe {
				let dest_ptr = self.svo.as_mut_ptr().add(self.elements * size);
				copy_nonoverlapping(v.ptr, dest_ptr, size * len);
			}
		} else {
			let copy_svo = self.capacity == 0 && self.elements != 0;
			if needed > self.capacity * page_size!() {
				if !self.resize_impl(needed) {
					return Err(err!(Alloc));
				}
			}
			if copy_svo {
				unsafe {
					copy_nonoverlapping(&self.svo as *const u8, self.ptr, self.elements * size);
				}
			}

			unsafe {
				let dest_ptr = self.ptr.add(self.elements * size);
				copy_nonoverlapping(v.ptr, dest_ptr, size * len);
			}
		}

		self.elements += len;
		Ok(())
	}

	pub fn resize(&mut self, n: usize) -> Result<(), Error> {
		let size = size_of::<T>();
		let needed = size * n;
		if needed > self.svo.len() {
			if !self.resize_impl(needed) {
				Err(err!(Alloc))
			} else {
				self.elements = n;
				Ok(())
			}
		} else {
			if self.capacity > 0 {
				unsafe {
					unmap(self.ptr, self.capacity);
				}
			}
			self.elements = n;
			self.capacity = 0;
			Ok(())
		}
	}

	pub fn len(&self) -> usize {
		self.elements
	}

	pub fn clear(&mut self) {
		self.elements = 0;
	}

	fn resize_impl(&mut self, needed: usize) -> bool {
		let pages = pages!(needed);
		let tmp = unsafe { map(pages) };
		if tmp.is_null() {
			false
		} else {
			let size = pages * page_size!();
			let cur = self.capacity * page_size!();

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
}
