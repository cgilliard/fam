use core::cell::UnsafeCell;
use core::marker::Sized;
use core::mem::size_of;
use core::ops::Deref;
use core::ops::Drop;
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

static mut SLAB_32: Option<SlabAllocator> = None;
static mut SLAB_96: Option<SlabAllocator> = None;
static mut SLAB_224: Option<SlabAllocator> = None;
static mut SLAB_480: Option<SlabAllocator> = None;
static mut SLAB_992: Option<SlabAllocator> = None;
static mut SLAB_2016: Option<SlabAllocator> = None;
static mut SLAB_4064: Option<SlabAllocator> = None;
static mut SLAB_INIT: Lock = Lock {
	state: UnsafeCell::new(0),
};

#[allow(static_mut_refs)]
fn get_slab_allocator(size: usize) -> Option<&'static mut SlabAllocator> {
	unsafe {
		let mut sa: Option<&mut SlabAllocator> = None;
		let mut lock = SLAB_INIT.read();
		if size <= 32 {
			sa = SLAB_32.as_mut();
		} else if size <= 96 {
			sa = SLAB_96.as_mut();
		} else if size <= 224 {
			sa = SLAB_224.as_mut();
		} else if size <= 480 {
			sa = SLAB_480.as_mut();
		} else if size <= 992 {
			sa = SLAB_992.as_mut();
		} else if size <= 2016 {
			sa = SLAB_2016.as_mut();
		} else if size <= 4064 {
			sa = SLAB_4064.as_mut();
		}

		if sa.is_none() && size < 4064 {
			lock.unlock();
			let _ = SLAB_INIT.write();

			if size <= 32 {
				SLAB_32 = crate::std::option::Option::Some(
					SlabAllocator::new(32, 0xFFFFFFFF, 0xFFFFFFFF, 20).unwrap(),
				);
				sa = SLAB_32.as_mut();
			} else if size <= 92 {
				SLAB_96 = crate::std::option::Option::Some(
					SlabAllocator::new(96, 0xFFFFFFFF, 0xFFFFFFFF, 20).unwrap(),
				);
				sa = SLAB_96.as_mut();
			} else if size <= 224 {
				SLAB_224 = crate::std::option::Option::Some(
					SlabAllocator::new(224, 0xFFFFFFFF, 0xFFFFFFFF, 20).unwrap(),
				);
				sa = SLAB_224.as_mut();
			} else if size <= 480 {
				SLAB_480 = crate::std::option::Option::Some(
					SlabAllocator::new(480, 0xFFFFFFFF, 0xFFFFFFFF, 20).unwrap(),
				);
				sa = SLAB_480.as_mut();
			} else if size <= 992 {
				SLAB_992 = crate::std::option::Option::Some(
					SlabAllocator::new(992, 0xFFFFFFFF, 0xFFFFFFFF, 20).unwrap(),
				);
				sa = SLAB_992.as_mut();
			} else if size <= 2016 {
				SLAB_2016 = crate::std::option::Option::Some(
					SlabAllocator::new(2016, 0xFFFFFFFF, 0xFFFFFFFF, 20).unwrap(),
				);
				sa = SLAB_2016.as_mut();
			} else {
				SLAB_4064 = crate::std::option::Option::Some(
					SlabAllocator::new(4064, 0xFFFFFFFF, 0xFFFFFFFF, 20).unwrap(),
				);
				sa = SLAB_4064.as_mut();
			}
		}

		sa
	}
}

pub struct Box<T: ?Sized> {
	pub value: *mut T,
	pub pages: usize,
	pub leak: bool,
	pub slab: Option<Slab>,
}

impl<T: ?Sized> Drop for Box<T> {
	fn drop(&mut self) {
		if !self.leak {
			let pages = self.pages;

			unsafe {
				ptr::drop_in_place(self.value);
				match self.slab {
					Some(mut slab) => match get_slab_allocator(slab.len()) {
						Some(sa) => {
							sa.free(&mut slab);
						}
						_ => {}
					},
					None => unmap(self.value as *mut u8, pages),
				}
			}
		}
	}
}

impl<T: Clone> Clone for Box<T> {
	fn clone(&self) -> Result<Self, Error> {
		let value = self.as_ref();
		match value.clone() {
			Ok(v) => Box::new(v),
			Err(e) => Err(e),
		}
	}
}

impl<T> Deref for Box<T>
where
	T: ?Sized,
{
	type Target = T;

	fn deref(&self) -> &Self::Target {
		unsafe { &*self.value }
	}
}

impl<T> Box<T> {
	pub fn new(t: T) -> Result<Self, Error> {
		let size = size_of::<T>();
		let pages = pages!(size);
		let sa = get_slab_allocator(size);

		let leak = false;
		let value;

		match sa {
			Some(sa) => {
				match sa.alloc() {
					Ok(slab) => {
						value = slab.get_raw() as *mut T;

						unsafe {
							ptr::write(value, t);
						}
						return Ok(Self {
							value,
							pages,
							leak,
							slab: Some(slab),
						});
					}
					Err(_e) => {
						// continue and try to call map below
					}
				}
			}
			None => {}
		}
		unsafe {
			value = map(pages) as *mut T;
			if value.is_null() {
				return Err(err!(Alloc));
			}
			ptr::write(value, t);
		}

		Ok(Self {
			value,
			pages,
			leak,
			slab: None,
		})
	}

	pub fn value(&self) -> *mut T {
		self.value
	}

	pub fn pages(&self) -> usize {
		self.pages
	}

	pub unsafe fn leak(&mut self) {
		self.leak = true;
	}

	pub fn as_ref(&self) -> &T {
		unsafe { &*self.value }
	}

	pub fn as_mut(&mut self) -> &mut T {
		unsafe { &mut *self.value }
	}
}

#[macro_export]
macro_rules! box_dyn {
	($type:expr, $trait:ident) => {{
		match Box::new($type) {
			Ok(mut boxv) => {
				unsafe {
					boxv.leak();
				}
				let boxv_dyn: Box<dyn $trait> = Box {
					value: boxv.value(),
					pages: boxv.pages(),
					leak: false,
					slab: boxv.slab.clone().unwrap(),
				};
				Ok(boxv_dyn)
			}
			Err(e) => Err(e),
		}
	}};
}

#[cfg(test)]
mod test {
	use std::boxed::Box;
	use std::clone::Clone;
	use std::result::{Result::Err, Result::Ok};

	#[test]
	fn test_box() {
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
	fn test_dyn() {
		let t = TestSample { data: 1 };
		let mut sample = Box::new(t).unwrap();
		unsafe {
			sample.leak();
		}
		let sample_b: Box<dyn GetData> = Box {
			value: sample.value,
			pages: sample.pages,
			leak: false,
			slab: sample.slab.clone().unwrap(),
		};

		assert_eq!(sample_b.get_data(), 1);

		// create a dynamic dispatch to trait GetData
		let v: Box<dyn GetData> = match box_dyn!(TestSample { data: 2 }, GetData) {
			Ok(v) => v,
			Err(_e) => exit!("box_dyn failed!"),
		};
		assert_eq!(v.get_data(), 2);
	}
}
