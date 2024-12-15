use core::intrinsics::{unchecked_div, unchecked_rem};
use core::ptr::copy_nonoverlapping;

pub fn u32_to_str(num: u32) -> &'static str {
	match num {
		0 => "0",
		1 => "1",
		2 => "2",
		3 => "3",
		4 => "4",
		5 => "5",
		6 => "6",
		7 => "7",
		8 => "8",
		9 => "9",
		10 => "10",
		11 => "11",
		12 => "12",
		13 => "13",
		14 => "14",
		15 => "15",
		16 => "16",
		17 => "17",
		18 => "18",
		19 => "19",
		20 => "20",
		21 => "21",
		22 => "22",
		23 => "23",
		24 => "24",
		_ => "unknown", // Handle numbers outside the range 0-9
	}
}

pub fn strcmp(a: &str, b: &str) -> i32 {
	let len = if a.len() > b.len() { b.len() } else { a.len() };
	let x = a.as_bytes();
	let y = b.as_bytes();

	for i in 0..len {
		if x[i] != y[i] {
			return if x[i] > y[i] { 1 } else { -1 };
		}
	}

	if a.len() < b.len() {
		1
	} else if a.len() > b.len() {
		-1
	} else {
		0
	}
}

pub fn copy_slice(src: &[u8], dest: &mut [u8], len: usize) {
	unsafe {
		copy_nonoverlapping(src.as_ptr(), dest.as_mut_ptr(), len);
	}
}

pub fn divide_usize(n: usize, d: usize) -> usize {
	unsafe { unchecked_div(n, d) }
}

pub fn rem_usize(n: usize, d: usize) -> usize {
	unsafe { unchecked_rem(n, d) }
}

#[cfg(test)]
mod test {
	use super::strcmp;

	#[test]
	fn test_strcmp() {
		assert_eq!(strcmp("abc", "abc"), 0);
		assert_eq!(strcmp("abc", "def"), -1);
		assert_eq!(strcmp("def", "abc"), 1);
	}
}
