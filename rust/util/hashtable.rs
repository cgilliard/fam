use core::ops::{Deref, DerefMut};
use core::ptr::null_mut;
use prelude::*;

pub struct Node<V: PartialEq> {
	next: Ptr<Node<V>>,
	value: V,
}

impl<V: PartialEq> PartialEq for Node<V> {
	fn eq(&self, other: &Node<V>) -> bool {
		self.value == other.value
	}
}

impl<V: PartialEq> Deref for Node<V> {
	type Target = V;

	fn deref(&self) -> &Self::Target {
		&self.value
	}
}

impl<V: PartialEq> DerefMut for Node<V> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.value
	}
}

impl<V: PartialEq> Node<V> {
	pub fn new(value: V) -> Self {
		Self {
			next: Ptr::new(null_mut()),
			value,
		}
	}
}

pub struct Hashtable<V: PartialEq + Hash> {
	arr: Vec<Ptr<Node<V>>>,
}

impl<V: PartialEq + Hash> Hashtable<V> {
	pub fn new(size: usize) -> Result<Self, Error> {
		let mut arr = Vec::new();
		match arr.resize(size) {
			Ok(_) => Ok(Self { arr }),
			Err(e) => Err(e),
		}
	}

	pub fn insert(&mut self, mut node: Ptr<Node<V>>) -> bool {
		(*node).next = Ptr::null();
		let value = &*node;
		let index = value.hash() % self.arr.len();
		let mut ptr = self.arr[index];
		if ptr.is_null() {
			self.arr[index] = node;
		} else {
			let mut prev = Ptr::new(null_mut());
			while !ptr.is_null() {
				if *ptr == *value {
					return false;
				}
				prev = ptr;
				ptr = (*ptr).next;
			}

			(*prev).next = node;
		}
		true
	}

	pub fn find(&self, value: V) -> Option<Ptr<Node<V>>> {
		let mut ptr = self.arr[value.hash() % self.arr.len()];
		while !ptr.is_null() {
			if **ptr == value {
				return Some(Ptr::new(ptr.raw()));
			}
			ptr = (ptr.as_ref()).next;
		}
		None
	}

	pub fn remove(&mut self, value: V) -> Option<Ptr<Node<V>>> {
		let index = value.hash() % self.arr.len();
		let mut ptr = self.arr[index];

		if !ptr.is_null() && (*ptr).value == value {
			self.arr[index] = (*ptr).next;
			return Some(Ptr::new(ptr.raw()));
		}
		let mut prev = self.arr[index];

		while !ptr.is_null() {
			if (*ptr).value == value {
				(*prev).next = (*ptr).next;
				return Some(Ptr::new(ptr.raw()));
			}
			prev = ptr;
			ptr = (*ptr).next;
		}
		None
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::sys::alloc;
	use core::mem::size_of;
	use core::slice::from_raw_parts;
	use sys::getalloccount;

	struct TestValue {
		k: i32,
		v: i32,
	}

	impl PartialEq for TestValue {
		fn eq(&self, other: &TestValue) -> bool {
			self.k == other.k
		}
	}

	impl Hash for TestValue {
		fn hash(&self) -> usize {
			let slice =
				unsafe { from_raw_parts(&self.k as *const i32 as *const u8, size_of::<i32>()) };
			murmur3_32_of_slice(slice, MURMUR_SEED) as usize
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
			let node = Ptr::new(v);
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

		let v1 = Ptr::alloc(Node::new(TestValue { k: 1, v: 2 })).unwrap();
		let v2 = Ptr::alloc(Node::new(TestValue { k: 2, v: 3 })).unwrap();
		let v3 = Ptr::alloc(Node::new(TestValue { k: 3, v: 4 })).unwrap();

		let v4 = Ptr::alloc(Node::new(TestValue { k: 1, v: 2 })).unwrap();
		let v5 = Ptr::alloc(Node::new(TestValue { k: 2, v: 3 })).unwrap();
		let v6 = Ptr::alloc(Node::new(TestValue { k: 3, v: 4 })).unwrap();

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
