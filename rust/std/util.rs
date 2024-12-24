use core::intrinsics::{unchecked_div, unchecked_rem};
use core::ptr::copy_nonoverlapping;
use core::slice::from_raw_parts;
use prelude::*;

pub fn subslice<N>(n: &[N], off: usize, len: usize) -> Result<&[N], Error> {
	if len + off > n.len() {
		Err(ErrorKind::OutOfBounds.into())
	} else {
		Ok(unsafe { from_raw_parts(n.as_ptr().add(off), len) })
	}
}

pub fn u128_to_str(mut n: u128, offset: usize, buf: &mut [u8]) -> usize {
	let buf_len = buf.len();
	let mut i = buf_len - 1;

	while n > 0 {
		if i == 0 {
			break;
		}
		if i < buf_len {
			buf[i] = b'0' + (n % 10) as u8;
		}
		n /= 10;
		i -= 1;
	}
	let mut len = buf_len - i - 1;

	if len == 0 && buf_len > 0 && offset < buf_len {
		buf[offset] = b'0';
		len = 1;
	} else {
		let mut k = 0;
		for j in i + 1..buf_len {
			if k + offset < buf_len {
				buf[k + offset] = buf[j];
			}
			k += 1;
		}
	}
	len
}

pub fn i128_to_str(mut n: i128, buf: &mut [u8]) -> usize {
	if n < 0 {
		n *= -1;
		if buf.len() < 2 {
			0
		} else {
			buf[0] = b'-';
			u128_to_str(n as u128, 1, buf) + 1
		}
	} else {
		u128_to_str(n as u128, 0, buf)
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
