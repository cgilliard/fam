use core::ptr::null_mut;
use prelude::*;

pub struct RbTreeNodePair<V> {
	cur: Ptr<Node<V>>,
	parent: Ptr<Node<V>>,
	is_right: bool,
}

type RbTreeSearch<V> =
	fn(base: Ptr<Node<V>>, value: Ptr<Node<V>>, retval: RbTreeNodePair<V>) -> i32;

pub struct Node<V> {
	parent: Ptr<Node<V>>,
	right: Ptr<Node<V>>,
	left: Ptr<Node<V>>,
	value: V,
}

pub struct RbTree<V> {
	root: Ptr<Node<V>>,
}

impl<V> RbTree<V> {
	pub fn new() -> Self {
		Self {
			root: Ptr::new(null_mut()),
		}
	}

	pub fn insert(&mut self, n: Ptr<Node<V>>) {}

	pub fn remove(&mut self, n: V) -> Option<Ptr<Node<V>>> {
		None
	}
}
