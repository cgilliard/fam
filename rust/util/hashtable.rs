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

pub struct Node<V> {
	next: Pointer<Node<V>>,
	value: V,
}

impl<V> Node<V> {
	pub fn new(value: V) -> Self {
		Self {
			next: Pointer::new(null_mut()),
			value,
		}
	}

	pub fn value_mut(&mut self) -> &mut V {
		&mut self.value
	}

	pub fn value(&self) -> &V {
		&self.value
	}
}

pub struct Hashtable<V: Equal + Hash> {
	arr: Vec<Pointer<Node<V>>>,
}

impl<V: Equal + Hash> Hashtable<V> {
	pub fn new(size: usize) -> Result<Self, Error> {
		let mut arr = Vec::new();
		match arr.resize(size) {
			Ok(_) => Ok(Self { arr }),
			Err(e) => Err(e),
		}
	}

	pub fn insert(&mut self, mut node: Pointer<Node<V>>) -> bool {
		node.as_mut().next = Pointer::new(null_mut());
		let value = &node.as_ref().value;
		let index = value.hash() as usize % self.arr.len();
		let mut ptr = self.arr[index];
		let mut prev = Pointer::new(null_mut());

		while !ptr.raw().is_null() {
			if ptr.as_ref().value.equal(&value) {
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

	pub fn find(&self, value: V) -> Option<Pointer<Node<V>>> {
		let index = value.hash() as usize % self.arr.len();
		let mut ptr = self.arr[index];
		while !ptr.raw().is_null() {
			if (*(ptr.as_ref())).value.equal(&value) {
				return Some(Pointer::new(ptr.raw()));
			}
			ptr = (ptr.as_ref()).next;
		}
		None
	}

	pub fn remove(&mut self, value: V) -> Option<Pointer<Node<V>>> {
		let index = value.hash() as usize % self.arr.len();
		let mut ptr = self.arr[index];
		let mut prev = self.arr[index];
		let mut is_first = true;
		while !ptr.raw().is_null() {
			if (ptr.as_ref()).value.equal(&value) {
				if is_first {
					self.arr[index] = (*ptr.as_ref()).next;
				} else {
					(prev.as_mut()).next = (ptr.as_ref()).next;
				}
				return Some(Pointer::new(ptr.raw()));
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

/*

impl<K, V> Node<K, V> {
	pub fn new(key: K, value: V) -> Self {
		Self {
			next: Pointer::new(null_mut()),
			key,
			value,
		}
	}

	pub fn value_mut(&mut self) -> &mut V {
		&mut self.value
	}

	pub fn value(&self) -> &V {
		&self.value
	}
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

	pub fn get_ptr(&self, key: K) -> Option<Pointer<Node<K, V>>> {
		let index = key.hash() as usize % self.arr.len();
		let mut ptr = self.arr[index];
		while !ptr.raw().is_null() {
			if ptr.as_ref().key.equal(&key) {
				return Some(ptr);
			}
			ptr = (ptr.as_ref()).next;
		}
		None
	}

	pub fn get_ptr_mut(&mut self, key: K) -> Option<Pointer<Node<K, V>>> {
		let index = key.hash() as usize % self.arr.len();
		let mut ptr = self.arr[index];
		while !ptr.raw().is_null() {
			if ptr.as_ref().key.equal(&key) {
				return Some(ptr);
			}
			ptr = (ptr.as_ref()).next;
		}
		None
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

*/

#[cfg(test)]
mod test {
	use super::*;
	use crate::sys::{alloc, release};
	use sys::getalloccount;

	struct TestValue {
		k: i32,
		v: i32,
	}

	impl Hash for TestValue {
		fn hash(&self) -> u32 {
			let slice =
				unsafe { from_raw_parts(&self.k as *const i32 as *const u8, size_of::<i32>()) };
			murmur3_32_of_slice(slice, MURMUR_SEED)
		}
	}

	impl Equal for TestValue {
		fn equal(&self, other: &Self) -> bool {
			self.k == other.k
		}
	}

	impl From<i32> for TestValue {
		fn from(k: i32) -> Self {
			Self { k, v: 0 }
		}
	}

	#[test]
	fn test_hashtable1() {
		let initial = unsafe { getalloccount() };
		let v;
		unsafe {
			v = alloc(size_of::<Node<TestValue>>()) as *mut Node<TestValue>;
			*v = Node::new(TestValue { k: 1i32, v: 2i32 });
		}
		{
			let mut hash = Hashtable::new(1024).unwrap();
			let node = Pointer::new(v);
			hash.insert(node);

			let mut n = hash.find(1i32.into()).unwrap();
			assert_eq!(n.as_ref().value.v, 2);
			(n.as_mut()).value.v = 3i32;
			assert!(hash.find(4i32.into()).is_none());
			let n = hash.find(1i32.into()).unwrap();
			assert_eq!(n.as_ref().value.v, 3);
			let n = hash.remove(1i32.into()).unwrap();
			assert_eq!(n.as_ref().value.v, 3);
			unsafe {
				release(n.raw() as *mut u8);
			}
			assert!(hash.remove(1i32.into()).is_none());
		}
		assert_eq!(unsafe { getalloccount() }, initial);
	}
	/*
	#[test]
	fn test_hashtable_collisions() {
		let initial = unsafe { getalloccount() };
		let (v1, v2, v3, v4, v5, v6);
		unsafe {
			v1 = alloc(size_of::<Node<i32, i32>>()) as *mut Node<i32, i32>;
			*v1 = Node::new(1i32, 2i32);

			v2 = alloc(size_of::<Node<i32, i32>>()) as *mut Node<i32, i32>;
			*v2 = Node::new(2i32, 3i32);

			v3 = alloc(size_of::<Node<i32, i32>>()) as *mut Node<i32, i32>;
			*v3 = Node::new(3i32, 4i32);

			v4 = alloc(size_of::<Node<i32, i32>>()) as *mut Node<i32, i32>;
			*v4 = Node::new(1i32, 2i32);

			v5 = alloc(size_of::<Node<i32, i32>>()) as *mut Node<i32, i32>;
			*v5 = Node::new(2i32, 3i32);

			v6 = alloc(size_of::<Node<i32, i32>>()) as *mut Node<i32, i32>;
			*v6 = Node::new(3i32, 4i32);
		}

		{
			let mut hash = Hashtable::new(1).unwrap();
			assert!(hash.insert(Pointer::new(v1)));
			assert!(hash.insert(Pointer::new(v2)));
			assert!(hash.insert(Pointer::new(v3)));
			assert!(!hash.insert(Pointer::new(v4)));
			assert!(!hash.insert(Pointer::new(v5)));
			assert!(!hash.insert(Pointer::new(v6)));

			assert_eq!(hash.get_ptr(1i32).unwrap().as_ref().value(), &2);

			unsafe {
				release(v4 as *mut u8);
				release(v5 as *mut u8);
				release(v6 as *mut u8);
			}

			unsafe {
				let n = hash.get_mut(1i32).unwrap();
				assert_eq!((*n).value(), &2);
				*(*n).value_mut() = 3;
				assert!(hash.get(4i32).is_none());
				let n = hash.get(1i32).unwrap();
				assert_eq!((*n).value(), &3);

				let n = hash.get_mut(2i32).unwrap();
				assert_eq!((*n).value(), &3);
				// *(*n).value_mut() = 4;
				*hash.get_ptr_mut(1i32).unwrap().as_mut().value_mut() = 4;
				assert!(hash.get(4i32).is_none());
				let n = hash.get(2i32).unwrap();
				assert_eq!((*n).value(), &4);

				let n = hash.get_mut(3i32).unwrap();
				assert_eq!((*n).value(), &4);
				*(*n).value_mut() = 5;
				assert!(hash.get(4i32).is_none());
				let n = hash.get(3i32).unwrap();
				assert_eq!((*n).value(), &5);
			}

			let n = hash.remove(1i32).unwrap();

			//assert_eq!(ptr!(n as *mut Node<).as_ref().value(), &3);
			unsafe {
				release(n as *mut u8);
			}
			assert!(hash.remove(1i32).is_none());

			let n = hash.remove(2i32).unwrap();
			//assert_eq!((*n).value(), &4);
			unsafe {
				release(n as *mut u8);
			}
			assert!(hash.remove(2i32).is_none());

			let n = hash.remove(3i32).unwrap();
			//assert_eq!((*n).value(), &5);
			unsafe {
				release(n as *mut u8);
			}
			assert!(hash.remove(3i32).is_none());
		}
		assert_eq!(unsafe { getalloccount() }, initial);
	}
		*/
}
