use core::cmp::PartialEq;
use prelude::*;

#[derive(PartialEq, Ord, PartialOrd, Eq)]
struct TestPartialEq {
	x: u32,
}

#[no_mangle]
pub extern "C" fn real_main(_argc: i32, _argv: *const *const u8) -> i32 {
	let _ = unsafe { crate::sys::getalloccount() };
	let _ = unsafe { crate::sys::getpagesize() };

	let v = vec![1, 2, 3].unwrap();
	println!("v[0]={}", v[0]);

	let ptr = Ptr::alloc(1usize).unwrap();
	let ptr2 = Ptr::new(ptr.raw());
	if ptr == ptr2 {
		println!("they are equal");
	}

	let mut tree = RbTree::new();
	let node = Ptr::alloc(RbTreeNode::new(1i32)).unwrap();
	tree.insert(node, &mut move |base, value| {
		let is_right = false;
		let mut cur = base;
		let parent = Ptr::null();

		while !cur.is_null() {
			let _cmp = (*cur).value.compare(&(*value).value);
			if (*cur).value.compare(&(*value).value) == -1 {}
			cur = cur.left;
		}
		RbNodePair {
			cur,
			parent,
			is_right,
		}
	});

	let x1 = TestPartialEq { x: 1 };
	let x2 = TestPartialEq { x: 2 };
	let x3 = TestPartialEq { x: 1 };

	if x1 == x2 {
		println!("eq");
	} else {
		println!("ne");
	}
	if x1 == x3 {
		println!("eq2");
	}

	let err: Error = ErrorKind::Alloc.into();
	let err2: Error = ErrorKind::Alloc.into();
	if err == err2 {}

	println!("testing {}{}{}!", 1, 2, 3);
	unsafe {
		crate::sys::sleep_millis(1);
	}
	0
}
