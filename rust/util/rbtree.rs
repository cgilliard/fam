use core::ops::FnMut;
use prelude::*;

pub struct RbNodePair<V: Ord> {
	pub cur: Ptr<RbTreeNode<V>>,
	pub parent: Ptr<RbTreeNode<V>>,
	pub is_right: bool,
}

type RbTreeSearch<V> = dyn FnMut(Ptr<RbTreeNode<V>>, Ptr<RbTreeNode<V>>) -> RbNodePair<V>;

pub struct RbTreeNode<V: Ord> {
	pub parent: Ptr<RbTreeNode<V>>,
	pub right: Ptr<RbTreeNode<V>>,
	pub left: Ptr<RbTreeNode<V>>,
	is_red: bool,
	pub value: V,
}

enum Color {
	Black,
	Red,
}

impl<V: Ord> Display for RbTreeNode<V> {
	fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
		writeb!(
			*f,
			"Node: parent={},left={},right={},color={},bitcolor={}",
			self.parent,
			self.left,
			self.right,
			if self.is_red { "red" } else { "black" },
			if self.parent.get_bit() {
				"red"
			} else {
				"black"
			}
		)
	}
}

impl<V: Ord> RbTreeNode<V> {
	pub fn new(value: V) -> Self {
		let ret = Self {
			parent: Ptr::null(),
			right: Ptr::null(),
			left: Ptr::null(),
			is_red: true,
			value,
		};
		ret
	}

	fn set_color(&mut self, color: Color) {
		match color {
			Color::Black => {
				self.is_red = false;
			}
			Color::Red => {
				self.is_red = true;
			}
		}
	}

	fn is_root(&self) -> bool {
		self.parent.is_null()
	}

	fn is_red(&self) -> bool {
		self.is_red
	}

	fn is_black(&self) -> bool {
		!self.is_red()
	}
}

pub struct RbTree<V: Ord> {
	root: Ptr<RbTreeNode<V>>,
}

impl<V: Ord> RbTree<V> {
	pub fn new() -> Self {
		Self { root: Ptr::null() }
	}

	pub fn root(&self) -> Ptr<RbTreeNode<V>> {
		self.root
	}

	pub fn insert(
		&mut self,
		n: Ptr<RbTreeNode<V>>,
		search: &mut RbTreeSearch<V>,
	) -> Option<Ptr<RbTreeNode<V>>> {
		let pair = search(self.root, n);
		let ret = self.insert_impl(n, pair);
		if ret.is_none() {
			self.insert_fixup(n);
		}
		ret
	}

	pub fn remove(&mut self, _n: V) -> Option<Ptr<RbTreeNode<V>>> {
		None
	}

	fn insert_impl(
		&mut self,
		mut n: Ptr<RbTreeNode<V>>,
		mut pair: RbNodePair<V>,
	) -> Option<Ptr<RbTreeNode<V>>> {
		let ret = None;
		if pair.cur.is_null() {
			n.parent = pair.parent;
			if pair.parent.is_null() {
				self.root = n;
			} else {
				match pair.is_right {
					true => (*pair.parent).right = n,
					false => (*pair.parent).left = n,
				}
			}
		} else {
			// TODO: transplant
		}
		ret
	}

	fn rotate_left(&mut self, mut x: Ptr<RbTreeNode<V>>) {
		let mut y = x.right;
		x.right = y.left;
		if !y.left.is_null() {
			y.left.parent = x;
		}
		y.parent = x.parent;
		if x.parent.is_null() {
			self.root = y;
		} else if x == x.parent.left {
			x.parent.left = y;
		} else {
			x.parent.right = y;
		}
		y.left = x;
		x.parent = y;
	}

	fn rotate_right(&mut self, mut x: Ptr<RbTreeNode<V>>) {
		let mut y = x.left;
		x.left = y.right;
		if !y.right.is_null() {
			y.right.parent = x;
		}
		y.parent = x.parent;
		if x.parent.is_null() {
			self.root = y;
		} else if x == x.parent.right {
			x.parent.right = y;
		} else {
			x.parent.left = y;
		}
		y.right = x;
		x.parent = y;
	}

	fn insert_fixup(&mut self, mut k: Ptr<RbTreeNode<V>>) {
		let (mut parent, mut uncle, mut gparent);
		while !k.is_root() && k.parent.is_red() {
			parent = k.parent;
			gparent = parent.parent;
			if parent == gparent.left {
				uncle = gparent.right;
				if !uncle.is_null() && uncle.is_red() {
					parent.set_color(Color::Black);
					uncle.set_color(Color::Black);
					gparent.set_color(Color::Red);
					k = gparent
				} else {
					if k == parent.right {
						k = k.parent;
						self.rotate_left(k);
					}
					parent = k.parent;
					gparent = parent.parent;
					parent.set_color(Color::Black);
					gparent.set_color(Color::Red);
					self.rotate_right(gparent);
				}
			} else {
				uncle = gparent.left;
				if !uncle.is_null() && uncle.is_red() {
					parent.set_color(Color::Black);
					uncle.set_color(Color::Black);
					gparent.set_color(Color::Red);
					k = gparent;
				} else {
					if k == parent.left {
						k = k.parent;
						self.rotate_right(k);
					}
					parent = k.parent;
					gparent = parent.parent;
					parent.set_color(Color::Black);
					gparent.set_color(Color::Red);
					self.rotate_left(gparent);
				}
			}
		}
		self.root.set_color(Color::Black);
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use util::murmur::murmur3_32_of_u64;

	fn validate_node(
		node: Ptr<RbTreeNode<u64>>,
		mut black_count: Ptr<i32>,
		mut current_black_count: i32,
	) {
		if node.is_null() {
			if *black_count == 0 {
				*black_count = current_black_count;
			} else {
				assert_eq!(current_black_count, *black_count);
			}
			return;
		}

		if node.is_black() {
			current_black_count += 1;
		} else {
			if !node.parent.is_black() {
				println!("red/black violation node={}", node);
			}
			assert!(node.parent.is_black());
		}
		validate_node(node.right, black_count, current_black_count);
		validate_node(node.left, black_count, current_black_count);
	}

	fn validate_tree(root: Ptr<RbTreeNode<u64>>) {
		let black_count = Ptr::alloc(0).unwrap();
		assert!(root.is_black());
		validate_node(root, black_count, 0);
	}

	#[allow(dead_code)]
	fn print_node(node: Ptr<RbTreeNode<u64>>, depth: usize) {
		if node.is_null() {
			for _ in 0..depth {
				print!("    ");
			}
			println!("0 (B)");
			return;
		}

		print_node((*node).right, depth + 1);
		for _ in 0..depth {
			print!("    ");
		}
		println!(
			"{} {} ({})",
			node,
			node.value,
			if node.is_red() { "R" } else { "B" }
		);
		print_node((*node).left, depth + 1);
	}

	#[allow(dead_code)]
	fn print_tree(root: Ptr<RbTreeNode<u64>>) {
		if root.is_null() {
			println!("Red-Black Tree (root = 0) Empty Tree!");
		} else {
			println!("Red-Black Tree (root = {})", root);
			println!("===================================");
			print_node(root, 0);
			println!("===================================");
		}
	}

	#[test]
	fn test_rbtree1() {
		let mut tree = RbTree::new();

		let mut search = move |base: Ptr<RbTreeNode<u64>>, value: Ptr<RbTreeNode<u64>>| {
			let mut is_right = false;
			let mut cur = base;
			let mut parent = Ptr::null();

			while !cur.is_null() {
				let cmp = (*value).value.compare(&(*cur).value);
				if cmp == 0 {
					break;
				} else if cmp == -1 {
					parent = cur;
					is_right = false;
					cur = cur.left;
				} else {
					parent = cur;
					is_right = true;
					cur = cur.right;
				}
			}

			RbNodePair {
				cur,
				parent,
				is_right,
			}
		};

		let size = 300;
		let seed = 0x1234;
		for i in 0..size {
			let v = murmur3_32_of_u64(i, seed);
			let next = Ptr::alloc(RbTreeNode::new(v as u64)).unwrap();
			tree.insert(next, &mut search);
			//print_tree(tree.root());
			validate_tree(tree.root());
		}

		for i in 0..size {
			let v = murmur3_32_of_u64(i, seed);
			let ptr = Ptr::alloc(RbTreeNode::new(v as u64)).unwrap();
			let res = search(tree.root(), ptr);
			assert!(!res.cur.is_null());
			assert_eq!((*(res.cur)).value, v as u64);
			ptr.release();
		}
	}
}
