use core::mem::size_of;
use core::ptr::null_mut;
use core::slice::from_raw_parts;
use prelude::*;

pub trait Hash {
	fn hash(&self) -> u32;
}

pub trait Equal {
	fn equal(&self, other: &Self) -> bool;
}

pub struct Node<K, V> {
	next: Pointer<Node<K, V>>,
	key: K,
	value: V,
}

pub struct Hashtable<K: Equal + Hash, V> {
	arr: Vec<Pointer<Node<K, V>>>,
}

impl<K: Equal + Hash, V> Hashtable<K, V> {
	pub fn new(size: usize) -> Result<Self, Error> {
		let mut arr = Vec::new();
		match arr.resize(size) {
			Ok(_) => Ok(Self { arr }),
			Err(e) => Err(e),
		}
	}

	pub fn insert(&mut self, mut node: Pointer<Node<K, V>>) -> bool {
		node.as_mut().next = Pointer::new(null_mut());
		let key = &node.as_ref().key;
		let index = key.hash() as usize % self.arr.len();
		let mut ptr = self.arr[index];
		let mut prev = Pointer::new(null_mut());
		while !ptr.raw().is_null() {
			if ptr.as_ref().key.equal(&key) {
				return false;
			}
			prev = ptr;
			ptr = ptr.as_ref().next;
		}

		if prev.raw().is_null() {
			self.arr[index] = node;
		} else {
			prev.as_mut().next = node;
		}
		true
	}

	pub fn get(&self, key: K) -> Option<*const Node<K, V>> {
		let index = key.hash() as usize % self.arr.len();
		let mut ptr = self.arr[index];
		while !ptr.raw().is_null() {
			if ptr.as_ref().key.equal(&key) {
				return Some(ptr.raw());
			}
			ptr = (ptr.as_ref()).next;
		}
		None
	}

	pub fn get_mut(&self, key: K) -> Option<*mut Node<K, V>> {
		let index = key.hash() as usize % self.arr.len();
		let mut ptr = self.arr[index];
		while !ptr.raw().is_null() {
			if ptr.as_ref().key.equal(&key) {
				return Some(ptr.raw());
			}
			ptr = (ptr.as_ref()).next;
		}
		None
	}

	pub fn remove(&mut self, key: K) -> Option<*mut Node<K, V>> {
		let index = key.hash() as usize % self.arr.len();
		let mut ptr = self.arr[index];
		let mut prev = self.arr[index];
		let mut is_first = true;
		while !ptr.raw().is_null() {
			if (ptr.as_ref()).key.equal(&key) {
				if is_first {
					self.arr[index] = (*ptr.as_ref()).next;
				} else {
					(prev.as_mut()).next = (ptr.as_ref()).next;
				}
				return Some(ptr.raw());
			}
			is_first = false;
			prev = ptr;
			ptr = (ptr.as_ref()).next;
		}
		None
	}
}

impl Hash for i32 {
	fn hash(&self) -> u32 {
		let slice = unsafe { from_raw_parts(&*self as *const i32 as *const u8, size_of::<i32>()) };
		murmur3_32_of_slice(slice, MURMUR_SEED)
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
	use crate::sys::{alloc, release};
	use sys::getalloccount;

	#[test]
	fn test_hashtable1() {
		let initial = unsafe { getalloccount() };
		unsafe {
			let v = alloc(size_of::<Node<i32, i32>>()) as *mut Node<i32, i32>;
			(*v).key = 1i32;
			(*v).value = 2i32;
			{
				let mut hash = Hashtable::new(1024).unwrap();
				let node = Pointer::new(v);
				hash.insert(node);
				let n = hash.get_mut(1i32).unwrap();
				assert_eq!((*n).value, 2);
				(*n).value = 3;
				assert!(hash.get(4i32).is_none());
				let n = hash.get(1i32).unwrap();
				assert_eq!((*n).value, 3);
				let n = hash.remove(1i32).unwrap();
				assert_eq!((*n).value, 3);
				release(n as *mut u8);
				assert!(hash.remove(1i32).is_none());
			}
		}
		assert_eq!(unsafe { getalloccount() }, initial);
	}

	#[test]
	fn test_hashtable_collisions() {
		let initial = unsafe { getalloccount() };
		unsafe {
			let v1 = alloc(size_of::<Node<i32, i32>>()) as *mut Node<i32, i32>;
			(*v1).key = 1i32;
			(*v1).value = 2i32;

			let v2 = alloc(size_of::<Node<i32, i32>>()) as *mut Node<i32, i32>;
			(*v2).key = 2i32;
			(*v2).value = 3i32;

			let v3 = alloc(size_of::<Node<i32, i32>>()) as *mut Node<i32, i32>;
			(*v3).key = 3i32;
			(*v3).value = 4i32;

			let v4 = alloc(size_of::<Node<i32, i32>>()) as *mut Node<i32, i32>;
			(*v4).key = 1i32;
			(*v4).value = 2i32;

			let v5 = alloc(size_of::<Node<i32, i32>>()) as *mut Node<i32, i32>;
			(*v5).key = 2i32;
			(*v5).value = 3i32;

			let v6 = alloc(size_of::<Node<i32, i32>>()) as *mut Node<i32, i32>;
			(*v6).key = 3i32;
			(*v6).value = 4i32;

			{
				let mut hash = Hashtable::new(1).unwrap();
				assert!(hash.insert(Pointer::new(v1)));
				assert!(hash.insert(Pointer::new(v2)));
				assert!(hash.insert(Pointer::new(v3)));
				assert!(!hash.insert(Pointer::new(v4)));
				assert!(!hash.insert(Pointer::new(v5)));
				assert!(!hash.insert(Pointer::new(v6)));

				release(v4 as *mut u8);
				release(v5 as *mut u8);
				release(v6 as *mut u8);

				let n = hash.get_mut(1i32).unwrap();
				assert_eq!((*n).value, 2);
				(*n).value = 3;
				assert!(hash.get(4i32).is_none());
				let n = hash.get(1i32).unwrap();
				assert_eq!((*n).value, 3);

				let n = hash.get_mut(2i32).unwrap();
				assert_eq!((*n).value, 3);
				(*n).value = 4;
				assert!(hash.get(4i32).is_none());
				let n = hash.get(2i32).unwrap();
				assert_eq!((*n).value, 4);

				let n = hash.get_mut(3i32).unwrap();
				assert_eq!((*n).value, 4);
				(*n).value = 5;
				assert!(hash.get(4i32).is_none());
				let n = hash.get(3i32).unwrap();
				assert_eq!((*n).value, 5);

				let n = hash.remove(1i32).unwrap();
				assert_eq!((*n).value, 3);
				release(n as *mut u8);
				assert!(hash.remove(1i32).is_none());

				let n = hash.remove(2i32).unwrap();
				assert_eq!((*n).value, 4);
				release(n as *mut u8);
				assert!(hash.remove(2i32).is_none());

				let n = hash.remove(3i32).unwrap();
				assert_eq!((*n).value, 5);
				release(n as *mut u8);
				assert!(hash.remove(3i32).is_none());
			}
		}
		assert_eq!(unsafe { getalloccount() }, initial);
	}
}
