use core::cell::UnsafeCell;
use core::marker::{Sized, Unsize};
use core::mem::size_of;
use core::ops::{CoerceUnsized, Deref, Drop};
use core::ptr;
use err;
use std::clone::Clone;
use std::error::{Error, ErrorKind::Alloc};
use std::lock::Lock;
use std::option::{Option, Option::None, Option::Some};
use std::result::{Result, Result::Err, Result::Ok};
use std::slabs::Slab;
use std::slabs::SlabAllocator;
use sys::{map, unmap};

struct SlabAllocators {
	sa32: Option<SlabAllocator>,
	sa96: Option<SlabAllocator>,
	sa224: Option<SlabAllocator>,
	sa480: Option<SlabAllocator>,
	sa992: Option<SlabAllocator>,
	sa2016: Option<SlabAllocator>,
	sa4064: Option<SlabAllocator>,
}

static mut SLABS: SlabAllocators = SlabAllocators {
	sa32: None,
	sa96: None,
	sa224: None,
	sa480: None,
	sa992: None,
	sa2016: None,
	sa4064: None,
};

static mut SLAB_INIT: Lock = Lock {
	state: UnsafeCell::new(0),
};

macro_rules! init_lock {
	($size:expr, $name:ident) => {{
		let _ = SLAB_INIT.write();
		if SLABS.$name.is_none() {
			SLABS.$name = match SlabAllocator::new($size, 0xFFFFFFFF, 0xFFFFFFFF, 20) {
				Ok(sa) => Some(sa),
				_ => None,
			};
		}
	}};
}

macro_rules! match_lock {
	($size:expr, $name:ident, $r:expr) => {{
		match SLABS.$name.as_mut() {
			Some(_) => SLABS.$name.as_mut(),
			None => {
				$r.unlock();
				init_lock!($size, $name);
				SLABS.$name.as_mut()
			}
		}
	}};
}

macro_rules! cleanup_sa {
	($size:expr, $name:ident) => {{
		match SLABS.$name.as_mut() {
			Some(s) => {
				s.cleanup();
				SLABS.$name = None;
			}
			None => {}
		}
	}};
}

#[allow(static_mut_refs)]
pub unsafe fn cleanup_slab_allocators() {
	let _ = SLAB_INIT.write();
	cleanup_sa!(32, sa32);
	cleanup_sa!(96, sa96);
	cleanup_sa!(224, sa224);
	cleanup_sa!(480, sa480);
	cleanup_sa!(992, sa992);
	cleanup_sa!(2016, sa2016);
	cleanup_sa!(4064, sa4064);
}

#[allow(static_mut_refs)]
fn get_slab_allocator(size: usize) -> Option<&'static mut SlabAllocator> {
	unsafe {
		let mut r = SLAB_INIT.read();
		if size <= 32 {
			match_lock!(32, sa32, r)
		} else if size <= 96 {
			match_lock!(96, sa96, r)
		} else if size <= 224 {
			match_lock!(224, sa224, r)
		} else if size <= 480 {
			match_lock!(480, sa480, r)
		} else if size <= 992 {
			match_lock!(992, sa992, r)
		} else if size <= 2016 {
			match_lock!(2016, sa2016, r)
		} else if size <= 4064 {
			match_lock!(4064, sa4064, r)
		} else {
			None
		}
	}
}

const METADATA_TYPE_FLAG: u64 = 0x1 << 63;
const METADATA_LEAK_FLAG: u64 = 0x1 << 62;
const METADATA_SLAB_TYPE_FLAG1: u64 = 0x1 << 61;
const METADATA_SLAB_TYPE_FLAG2: u64 = 0x1 << 60;
const METADATA_SLAB_TYPE_FLAG3: u64 = 0x1 << 59;
const METADATA_SLABMASK: u64 =
	METADATA_SLAB_TYPE_FLAG1 | METADATA_SLAB_TYPE_FLAG2 | METADATA_SLAB_TYPE_FLAG3;
const METADATA_SLAB32_FLAG: u64 =
	METADATA_SLAB_TYPE_FLAG1 | METADATA_SLAB_TYPE_FLAG2 | METADATA_SLAB_TYPE_FLAG3;
const METADATA_SLAB96_FLAG: u64 = METADATA_SLAB_TYPE_FLAG1 | METADATA_SLAB_TYPE_FLAG2;
const METADATA_SLAB224_FLAG: u64 = METADATA_SLAB_TYPE_FLAG1 | METADATA_SLAB_TYPE_FLAG3;
const METADATA_SLAB480_FLAG: u64 = METADATA_SLAB_TYPE_FLAG2 | METADATA_SLAB_TYPE_FLAG3;
const METADATA_SLAB992_FLAG: u64 = METADATA_SLAB_TYPE_FLAG1;
const METADATA_SLAB2016_FLAG: u64 = METADATA_SLAB_TYPE_FLAG2;
const METADATA_SLAB4064_FLAG: u64 = METADATA_SLAB_TYPE_FLAG3;

enum MetaDataType {
	Mapped,
	Slab,
}

fn metadata_type(metadata: u64) -> MetaDataType {
	if metadata & METADATA_TYPE_FLAG != 0 {
		MetaDataType::Mapped
	} else {
		MetaDataType::Slab
	}
}

#[allow(static_mut_refs)]
fn metadata_slab_allocator(metadata: u64) -> Option<&'static mut SlabAllocator> {
	let mask = metadata & METADATA_SLABMASK;
	if mask == 0 {
		exit!("invalid slab allocator metadata!");
	} else if mask == METADATA_SLAB32_FLAG {
		unsafe { SLABS.sa32.as_mut() }
	} else if mask == METADATA_SLAB96_FLAG {
		unsafe { SLABS.sa96.as_mut() }
	} else if mask == METADATA_SLAB224_FLAG {
		unsafe { SLABS.sa224.as_mut() }
	} else if mask == METADATA_SLAB480_FLAG {
		unsafe { SLABS.sa480.as_mut() }
	} else if mask == METADATA_SLAB992_FLAG {
		unsafe { SLABS.sa992.as_mut() }
	} else if mask == METADATA_SLAB2016_FLAG {
		unsafe { SLABS.sa2016.as_mut() }
	} else if mask == METADATA_SLAB4064_FLAG {
		unsafe { SLABS.sa4064.as_mut() }
	} else {
		None
	}
}

fn metadata_flags_for(size: usize) -> u64 {
	if size <= 32 {
		METADATA_SLAB32_FLAG
	} else if size <= 96 {
		METADATA_SLAB96_FLAG
	} else if size <= 224 {
		METADATA_SLAB224_FLAG
	} else if size <= 480 {
		METADATA_SLAB480_FLAG
	} else if size <= 992 {
		METADATA_SLAB992_FLAG
	} else if size <= 2016 {
		METADATA_SLAB2016_FLAG
	} else if size <= 4064 {
		METADATA_SLAB4064_FLAG
	} else {
		0
	}
}

fn metadata_leak(metadata: u64) -> bool {
	metadata & METADATA_LEAK_FLAG != 0
}

fn metadata_id(metadata: u64) -> usize {
	metadata as usize & 0xFFFFFFFFFFFFusize
}

fn metadata_pages(metadata: u64) -> usize {
	metadata as usize & 0xFFFFFFFFFFFFusize
}

pub struct Box<T: ?Sized> {
	ptr: *mut T,
	metadata: u64,
}

impl<T: ?Sized> Drop for Box<T> {
	fn drop(&mut self) {
		if !metadata_leak(self.metadata) {
			match metadata_type(self.metadata) {
				MetaDataType::Mapped => unsafe {
					unmap(self.ptr as *mut u8, metadata_pages(self.metadata));
				},
				MetaDataType::Slab => {
					match metadata_slab_allocator(self.metadata) {
						Some(sa) => {
							let mut slab =
								Slab::from_raw(self.ptr as *mut u8, metadata_id(self.metadata));
							sa.free(&mut slab);
						}
						None => {
							// TODO: handle error
						}
					}
				}
			}
		}
	}
}

impl<T> Clone for Box<T> {
	fn clone(&self) -> Result<Self, Error> {
		Ok(Self {
			ptr: self.ptr,
			metadata: self.metadata,
		})
	}
}

impl<T> Deref for Box<T>
where
	T: ?Sized,
{
	type Target = T;

	fn deref(&self) -> &Self::Target {
		unsafe { &*self.ptr }
	}
}

impl<T> Box<T>
where
	T: ?Sized,
{
	pub unsafe fn from_raw(ptr: *mut T, metadata: u64) -> Box<T> {
		Box { ptr, metadata }
	}
}

impl<T, U> CoerceUnsized<Box<U>> for Box<T>
where
	T: Unsize<U> + ?Sized,
	U: ?Sized,
{
}

impl<T> Box<T> {
	pub fn new(t: T) -> Result<Self, Error> {
		let size = size_of::<T>();
		match get_slab_allocator(size) {
			Some(sa) => {
				match sa.alloc() {
					Ok(slab) => {
						let ptr = slab.get_raw() as *mut T;
						unsafe {
							ptr::write(ptr, t);
						}
						let metadata = slab.get_id() as u64 | metadata_flags_for(size);
						return Ok(Self { ptr, metadata });
					}
					Err(_) => {} // continue to try to map
				}
			}
			None => {}
		}

		let pages = pages!(size);
		let ptr = unsafe { map(pages) } as *mut T;

		if ptr.is_null() {
			Err(err!(Alloc))
		} else {
			unsafe {
				ptr::write(ptr, t);
			}
			let metadata = pages as u64 | METADATA_TYPE_FLAG;
			Ok(Self { ptr, metadata })
		}
	}

	pub unsafe fn leak(&mut self) {
		self.metadata |= METADATA_LEAK_FLAG;
	}

	pub fn metadata(&self) -> u64 {
		self.metadata
	}

	pub fn as_ref(&self) -> &T {
		unsafe { &*self.ptr }
	}

	pub fn as_mut(&mut self) -> &mut T {
		unsafe { &mut *self.ptr }
	}

	pub fn as_ptr(&self) -> *const T {
		self.ptr
	}

	pub fn as_mut_ptr(&mut self) -> *mut T {
		self.ptr
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use core::ops::Fn;

	#[test]
	fn test_box1() {
		{
			let mut x = Box::new(4).unwrap();
			let y = x.as_ref();
			assert_eq!(*y, 4);

			let z = x.as_mut();
			*z = 10;
			assert_eq!(*z, 10);
			let a = x.clone().unwrap();
			let b = a.as_ref();
			assert_eq!(*b, 10);
		}
		unsafe {
			cleanup_slab_allocators();
		}
	}

	trait GetData {
		fn get_data(&self) -> i32;
	}

	struct TestSample {
		data: i32,
	}

	impl GetData for TestSample {
		fn get_data(&self) -> i32 {
			self.data
		}
	}

	#[test]
	fn test_box2() {
		{
			let mut b1: Box<TestSample> = Box::new(TestSample { data: 1 }).unwrap();
			let metadata = b1.metadata();
			unsafe {
				b1.leak();
			}
			let b2: Box<dyn GetData> = unsafe { Box::from_raw(b1.as_mut_ptr(), metadata) };
			assert_eq!(b2.get_data(), 1);

			let b3: Box<dyn GetData> = Box::new(TestSample { data: 2 }).unwrap();
			assert_eq!(b3.get_data(), 2);

			let b4 = Box::new(|x| 5 + x).unwrap();
			assert_eq!(b4(5), 10);
		}
		unsafe {
			cleanup_slab_allocators();
		}
	}

	struct BoxTest<CLOSURE>
	where
		CLOSURE: Fn(i32) -> i32,
	{
		x: Box<dyn GetData>,
		y: Box<CLOSURE>,
	}

	#[test]
	fn test_box3() {
		{
			let x = BoxTest {
				x: Box::new(TestSample { data: 8 }).unwrap(),
				y: Box::new(|x| x + 4).unwrap(),
			};

			assert_eq!(x.x.get_data(), 8);
			assert_eq!((x.y)(14), 18);
		}
		unsafe {
			cleanup_slab_allocators();
		}
	}
}
