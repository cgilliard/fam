use core::intrinsics::{unchecked_div, unchecked_rem};
use core::ptr::copy_nonoverlapping;

pub fn u64_to_str(mut n: u64, buf: &mut [u8]) -> usize {
	let mut i = buf.len() - 1;

	while n > 0 {
		if i == 0 {
			break;
		}
		if i < buf.len() {
			buf[i] = b'0' + (n % 10) as u8;
		}
		n /= 10;
		i -= 1;
	}
	let mut len = buf.len() - i - 1;

	if len == 0 && buf.len() > 0 {
		buf[0] = b'0';
		len = 1;
	} else {
		let mut k = 0;
		for j in i + 1..buf.len() {
			if k < buf.len() {
				buf[k] = buf[j];
			}
			k += 1;
		}
	}
	len
}

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
	if d == 0 {
		panic!("divide by 0");
	}
	unsafe { unchecked_div(n, d) }
}

pub fn rem_usize(n: usize, d: usize) -> usize {
	if d == 0 {
		panic!("rem by 0");
	}
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
