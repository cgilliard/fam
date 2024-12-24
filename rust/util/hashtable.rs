use core::mem::size_of;
use core::ptr::null_mut;
use core::slice::from_raw_parts;
use prelude::*;
use util::murmur::murmur3_32_of_slice;

pub trait Hash {
	fn hash(&self) -> u32;
}

pub trait Equal {
	fn equal(&self, other: &Self) -> bool;
}

struct Node<K, V> {
	next: *mut Node<K, V>,
	key: K,
	value: V,
}

pub struct Hashtable<K: Equal + Hash, V: Clone> {
	arr: Vec<*mut Node<K, V>>,
}

impl<K: Equal + Hash, V: Clone> Hashtable<K, V> {
	pub fn new(size: usize) -> Result<Self, Error> {
		let mut arr = Vec::new();
		match arr.resize(size) {
			Ok(_) => Ok(Self { arr }),
			Err(e) => Err(e),
		}
	}
	pub fn insert(&mut self, key: K, value: V) -> Result<(), Error> {
		let index = key.hash() as usize % self.arr.len();
		let mut ptr = self.arr[index];
		let mut prev = null_mut();
		while !ptr.is_null() {
			prev = ptr;
			ptr = unsafe { (*ptr).next };
		}
		match Box::new(Node {
			next: null_mut(),
			key,
			value,
		}) {
			Ok(mut b) => {
				unsafe {
					b.leak();
					if prev.is_null() {
						self.arr[index] = b.as_mut_ptr();
					} else {
						(*prev).next = b.as_mut_ptr();
					}
				}
				Ok(())
			}
			Err(e) => Err(e),
		}
	}

	pub fn get(&self, key: K) -> Result<Option<&V>, Error> {
		let index = key.hash() as usize % self.arr.len();
		let mut ptr = self.arr[index];
		unsafe {
			while !ptr.is_null() {
				if (*ptr).key.equal(&key) {
					return Ok(Some(&(*ptr).value));
				}
				ptr = (*ptr).next;
			}
		}
		Ok(None)
	}

	pub fn get_mut(&mut self, key: K) -> Result<Option<&mut V>, Error> {
		let index = key.hash() as usize % self.arr.len();
		let mut ptr = self.arr[index];
		unsafe {
			while !ptr.is_null() {
				if (*ptr).key.equal(&key) {
					return Ok(Some(&mut (*ptr).value));
				}
				ptr = (*ptr).next;
			}
		}
		Ok(None)
	}

	pub fn remove(&mut self, key: K) -> Result<Option<V>, Error> {
		let index = key.hash() as usize % self.arr.len();
		let mut ptr = self.arr[index];
		let mut prev = self.arr[index];
		let mut is_first = true;
		unsafe {
			while !ptr.is_null() {
				if (*ptr).key.equal(&key) {
					match (*ptr).value.clone() {
						Ok(ret) => {
							if is_first {
								self.arr[index] = (*ptr).next;
							} else {
								(*prev).next = (*ptr).next;
							}
							// free boxed resource
							let _b = Box::from_raw(ptr);
							return Ok(Some(ret));
						}
						Err(e) => return Err(e),
					}
				}
				is_first = false;
				prev = ptr;
				ptr = (*ptr).next;
			}
		}
		Ok(None)
	}

	pub fn remove_no_clone(&mut self, key: K) -> Option<()> {
		let index = key.hash() as usize % self.arr.len();
		let mut ptr = self.arr[index];
		let mut prev = self.arr[index];
		let mut is_first = true;
		unsafe {
			while !ptr.is_null() {
				if (*ptr).key.equal(&key) {
					if is_first {
						self.arr[index] = (*ptr).next;
					} else {
						(*prev).next = (*ptr).next;
					}
					let _b = Box::from_raw(ptr);
					return Some(());
				}
				is_first = false;
				prev = ptr;
				ptr = (*ptr).next;
			}
		}
		None
	}
}

impl Hash for i32 {
	fn hash(&self) -> u32 {
		let slice = unsafe { from_raw_parts(&*self as *const i32 as *const u8, size_of::<i32>()) };
		murmur3_32_of_slice(slice, 1)
	}
}
impl Equal for i32 {
	fn equal(&self, other: &Self) -> bool {
		*self == *other
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn test_hashtable() {
		let mut hash = Hashtable::new(1024).unwrap();
		assert!(hash.insert(1i32, 2i32).is_ok());
		assert_eq!(hash.get(1i32).unwrap().unwrap(), &2i32);
		assert!(hash.get(2i32).unwrap().is_none());

		*hash.get_mut(1i32).unwrap().unwrap() = 3i32;
		assert_eq!(hash.get(1i32).unwrap().unwrap(), &3i32);
		assert_eq!(hash.remove(1i32).unwrap().unwrap(), 3i32);
		assert!(hash.remove_no_clone(1i32).is_none());
	}
}
