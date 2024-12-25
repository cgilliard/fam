use core::mem::size_of;
use core::ops::{Deref, DerefMut};
use core::ptr::null_mut;
use core::slice::from_raw_parts;
use prelude::*;

pub trait Hash {
	fn hash(&self) -> usize;
}

pub trait Equal {
	fn equal(&self, other: &Self) -> bool;
}

pub struct Node<V> {
	next: Pointer<Node<V>>,
	value: V,
}

impl<T> Deref for Node<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&self.value
	}
}

impl<T> DerefMut for Node<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.value
	}
}

impl<V> Node<V> {
	pub fn new(value: V) -> Self {
		Self {
			next: Pointer::new(null_mut()),
			value,
		}
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
		let index = value.hash() % self.arr.len();
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
		let index = value.hash() % self.arr.len();
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
		let index = value.hash() % self.arr.len();
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

#[cfg(test)]
mod test {
	use super::*;
	use crate::sys::alloc;
	use sys::getalloccount;

	struct TestValue {
		k: i32,
		v: i32,
	}

	impl Hash for TestValue {
		fn hash(&self) -> usize {
			let slice =
				unsafe { from_raw_parts(&self.k as *const i32 as *const u8, size_of::<i32>()) };
			murmur3_32_of_slice(slice, MURMUR_SEED) as usize
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
			assert_eq!((*n).v, 2);
			(*n).v = 3i32;
			assert!(hash.find(4i32.into()).is_none());
			let n = hash.find(1i32.into()).unwrap();
			assert_eq!((*n).v, 3);
			let n = hash.remove(1i32.into()).unwrap();
			assert_eq!((*n).v, 3);
			n.release();
			assert!(hash.remove(1i32.into()).is_none());
		}
		assert_eq!(unsafe { getalloccount() }, initial);
	}

	#[test]
	fn test_hashtable_collisions() {
		let initial = unsafe { getalloccount() };

		let v1 = Pointer::alloc(Node::new(TestValue { k: 1, v: 2 })).unwrap();
		let v2 = Pointer::alloc(Node::new(TestValue { k: 2, v: 3 })).unwrap();
		let v3 = Pointer::alloc(Node::new(TestValue { k: 3, v: 4 })).unwrap();

		let v4 = Pointer::alloc(Node::new(TestValue { k: 1, v: 2 })).unwrap();
		let v5 = Pointer::alloc(Node::new(TestValue { k: 2, v: 3 })).unwrap();
		let v6 = Pointer::alloc(Node::new(TestValue { k: 3, v: 4 })).unwrap();

		{
			let mut hash = Hashtable::new(1).unwrap();
			assert!(hash.insert(v1));
			assert!(hash.insert(v2));
			assert!(hash.insert(v3));
			assert!(!hash.insert(v4));
			assert!(!hash.insert(v5));
			assert!(!hash.insert(v6));

			assert_eq!(hash.find(1i32.into()).unwrap().v, 2);
			assert!(hash.remove(4i32.into()).is_none());

			v4.release();
			v5.release();
			v6.release();

			let mut n = hash.find(1i32.into()).unwrap();
			assert_eq!((*n).v, 2);
			(*n).v = 3;
			let n = hash.find(1i32.into()).unwrap();
			assert_eq!((*n).v, 3);

			let mut n = hash.find(2i32.into()).unwrap();
			assert_eq!((*n).v, 3);
			(*n).v = 4;
			let n = hash.find(2i32.into()).unwrap();
			assert_eq!((*n).v, 4);

			let mut n = hash.find(3i32.into()).unwrap();
			assert_eq!((*n).v, 4);
			(*n).v = 5;
			let n = hash.find(3i32.into()).unwrap();
			assert_eq!((*n).v, 5);

			let n = hash.remove(1i32.into()).unwrap();
			assert_eq!((*n).v, 3);
			assert!(hash.remove(1i32.into()).is_none());

			n.release();

			let n = hash.remove(2i32.into()).unwrap();
			assert_eq!((*n).v, 4);
			assert!(hash.remove(2i32.into()).is_none());

			n.release();

			let n = hash.remove(3i32.into()).unwrap();
			assert_eq!((*n).v, 5);
			assert!(hash.remove(3i32.into()).is_none());
			n.release();
		}
		assert_eq!(unsafe { getalloccount() }, initial);
	}
}
