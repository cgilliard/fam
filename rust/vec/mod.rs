use base::sys::{getpagesize, map, unmap};
use core::marker::PhantomData;
use core::mem::size_of;
use core::ops::{Index, IndexMut};
use core::ptr::{copy_nonoverlapping, null_mut};
use error::{Error, ErrorKind::*};

macro_rules! pages {
	($v:expr) => {{
		let size = unsafe { getpagesize() };
		1 + ($v - 1) / size as u64
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
		unsafe { &*((self.ptr.add(index * core::mem::size_of::<T>())) as *const T) }
	}
}

impl<T> IndexMut<usize> for Vec<T> {
	fn index_mut(&mut self, index: usize) -> &mut Self::Output {
		if index >= self.elements as usize {
			panic!("array index out of bounds!");
		}
		unsafe { &mut *((self.ptr.add(index * core::mem::size_of::<T>())) as *mut T) }
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
				return err!(Alloc);
			}
		}

		unsafe {
			let dest_ptr = self.ptr.add((self.elements * size) as usize);
			copy_nonoverlapping(&v as *const T as *const u8, dest_ptr, size as usize);
		}

		self.elements += 1;

		Ok(())
	}

	fn resize_impl(&mut self, needed: u64) -> bool {
		let pages = pages!(needed);
		let tmp = unsafe { map(pages) };
		if tmp.is_null() {
			false
		} else {
			let size = (pages * page_size!()) as usize;
			unsafe {
				if self.capacity > 0 {
					copy_nonoverlapping(tmp, self.ptr, size);
				}
			}
			self.ptr = tmp;
			self.capacity = pages;
			true
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
	}
}
