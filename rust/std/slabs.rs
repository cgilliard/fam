use core::marker::Send;
use core::marker::Sync;
use core::mem::size_of;
use core::ptr;
use core::ptr::null_mut;
use core::slice::{from_raw_parts, from_raw_parts_mut};
use prelude::*;
use std::bitmap::BitMap;
use std::result::{Result, Result::Err, Result::Ok};
use sys::{ctz, map, unmap};

macro_rules! shift_per_level {
	() => {{
		unsafe { ctz(crate::sys::getpagesize() as u32) - 3 }
	}};
}

macro_rules! offset {
	($z:expr, $slab_size:expr) => {{
		let entries = 0x1
			<< unsafe {
				ctz(divide_usize(crate::sys::getpagesize() as usize, $slab_size) as u32) as usize
			};
		rem_usize($z, entries) * $slab_size
	}};
}

macro_rules! mask {
	() => {{
		(0x1 << shift_per_level!()) - 1
	}};
}

pub struct SlabAllocator {
	data: *mut *mut *mut *mut u8,
	head: *mut u8,
	tail: *mut u8,
	bitmap: BitMap,
	slab_struct_size: usize,
	slab_size: usize,
	max_free_slabs: u64,
	max_total_slabs: u64,
	lock: Lock,
	free_slabs: u64,
	total_slabs: u64,
}

unsafe impl Send for SlabAllocator {}
unsafe impl Sync for SlabAllocator {}

#[derive(Copy, Clone)]
pub struct Slab {
	data: *mut u8,
	next: *mut Slab,
	id: usize,
	len: usize,
}

const RESERVED: Slab = Slab {
	data: null_mut(),
	next: null_mut(),
	id: 0,
	len: 0,
};
const RESERVED_PTR: *const Slab = &RESERVED;

impl Clone for Slab {
	fn clone(&self) -> Result<Self, Error> {
		Ok(Self {
			data: self.data,
			next: self.next,
			id: self.id,
			len: self.len,
		})
	}
}

impl Slab {
	pub fn new(ptr: *mut Slab, len: usize, id: usize) -> Self {
		let data_ptr = unsafe { (ptr as *mut u8).add(size_of::<Slab>()) };

		let ret = Slab {
			data: data_ptr,
			next: RESERVED_PTR as *mut Slab,
			id,
			len,
		};
		unsafe {
			ptr::write(ptr, ret);
		}
		ret
	}

	pub fn get(&self) -> &[u8] {
		unsafe { from_raw_parts(self.data, self.len) }
	}

	pub fn get_mut(&mut self) -> &mut [u8] {
		unsafe { from_raw_parts_mut(self.data, self.len) }
	}

	pub fn get_raw(&self) -> *mut u8 {
		self.data
	}

	pub fn get_id(&self) -> usize {
		self.id
	}

	pub fn len(&self) -> usize {
		self.len
	}

	pub fn from_raw(data: *mut u8, id: usize) -> Self {
		let mut ret = Self {
			data,
			id,
			next: null_mut(),
			len: 0,
		};
		let slab_next_ptr = &mut ret.next as *mut *mut Slab as *mut u64;
		let reserved_ptr = RESERVED_PTR as *mut Slab;
		astore!(
			slab_next_ptr,
			*(&reserved_ptr as *const *mut Slab as *const u64)
		);
		ret
	}
}

impl SlabAllocator {
	pub fn new(
		slab_size: usize,
		max_free_slabs: u64,
		max_total_slabs: u64,
		bitmap_pages: usize,
	) -> Result<Self, Error> {
		let slab_struct_size = slab_size + size_of::<Slab>();

		if slab_struct_size > page_size!()
			|| (slab_struct_size != 0 && page_size!() % slab_struct_size != 0)
		{
			return Err(ErrorKind::IllegalArgument.into());
		}

		let bitmap = match BitMap::new(bitmap_pages) {
			Ok(bitmap) => bitmap,
			Err(e) => return Err(e),
		};

		let mut ret = Self {
			data: null_mut(),
			head: null_mut(),
			tail: null_mut(),
			bitmap,
			slab_size,
			slab_struct_size,
			max_free_slabs,
			max_total_slabs,
			lock: Lock::new(),
			free_slabs: 1,
			total_slabs: 0,
		};
		ret.head = match ret.grow() {
			Ok(s) => s.data,
			Err(e) => return Err(e),
		};
		ret.tail = ret.head;
		Ok(ret)
	}

	pub fn free_slabs(&self) -> u64 {
		aload!(&self.free_slabs)
	}

	pub fn total_slabs(&self) -> u64 {
		aload!(&self.total_slabs)
	}

	pub fn cleanup(&mut self) {
		let page_size = page_size!();
		unsafe {
			if !self.data.is_null() {
				let mut i = 0;
				while i < (page_size / size_of::<*mut u8>()) && !(*self.data.add(i)).is_null() {
					let mut j = 0;
					while j < (page_size / size_of::<*mut u8>())
						&& !(*(*self.data.add(i)).add(j)).is_null()
					{
						let mut k = 0;
						while k < (page_size / size_of::<*mut u8>())
							&& !(*(*(*self.data.add(i)).add(j)).add(k)).is_null()
						{
							unmap(*(*(*self.data.add(i)).add(j)).add(k), 1);
							k += 1;
						}
						unmap(*(*self.data.add(i)).add(j) as *mut u8, 1);
						j += 1;
					}
					unmap(*self.data.add(i) as *mut u8, 1);
					i += 1;
				}
				unmap(self.data as *mut u8, 1);
			}
		}
		self.bitmap.cleanup();
	}

	pub fn free(&mut self, slab: &mut Slab) {
		let slab_next_ptr = &mut slab.next as *mut *mut Slab as *mut u64;
		let reserved_ptr = RESERVED_PTR as *mut Slab;

		if !cas!(
			slab_next_ptr,
			&reserved_ptr as *const *mut Slab as *const u64,
			0u64
		) {
			exit!("double free attempt!");
		}

		if aadd!(&mut self.free_slabs, 1) > self.max_free_slabs {
			asub!(&mut self.free_slabs, 1);
			self.bitmap.free(slab.id);
			return;
		}
		loop {
			let tail = self.tail;
			let next_ptr: *mut *mut Slab = unsafe { &mut (*(self.tail as *mut Slab)).next };

			if tail == self.tail {
				let tail_next_ptr: *mut *mut Slab =
					unsafe { &mut (*(self.tail as *mut Slab)).next };

				if cas!(
					tail_next_ptr as *mut u64,
					next_ptr as *const u64,
					slab.data as u64
				) {
					let tail_ptr: *mut *mut u8 = &mut self.tail;
					let tail_ref_ptr: *const *mut u8 = &tail;

					cas!(
						tail_ptr as *mut u64,
						tail_ref_ptr as *const u64,
						slab.data as u64
					);

					break;
				}
			}
		}
	}

	pub fn alloc(&mut self) -> Result<Slab, Error> {
		let mut ret;
		loop {
			let head = self.head;
			let tail = self.tail;
			let next = unsafe { (*(self.head as *mut Slab)).next };

			if head == self.head {
				if head == tail {
					return self.grow();
				} else {
					ret = head;
					let head_ptr: *mut *mut u8 = &mut self.head;
					let head_ref_ptr: *const *mut u8 = &head;
					if cas!(
						head_ptr as *mut u64,
						head_ref_ptr as *const u64,
						next as u64
					) {
						break;
					}
				}
			}
		}

		asub!(&mut self.free_slabs, 1);
		unsafe {
			let mut slab = *((ret as *mut u8).sub(size_of::<Slab>()) as *mut Slab);
			slab.next = RESERVED_PTR as *mut Slab;
			Ok(slab)
		}
	}

	fn grow(&mut self) -> Result<Slab, Error> {
		if aadd!(&mut self.total_slabs, 1) > self.max_total_slabs {
			asub!(&mut self.total_slabs, 1);
			return Err(ErrorKind::CapacityExceeded.into());
		}

		let id = match self.bitmap.allocate() {
			Ok(id) => id,
			Err(e) => match e.kind {
				ErrorKind::CapacityExceeded => match self.bitmap.extend() {
					Ok(_) => match self.bitmap.allocate() {
						Ok(id) => id,
						Err(e) => {
							return Err(e);
						}
					},
					Err(e) => {
						return Err(e);
					}
				},
				_ => {
					return Err(e);
				}
			},
		};

		let page_size = page_size!();
		let next = id >> unsafe { ctz(divide_usize(page_size, self.slab_struct_size) as u32) };
		let k = next & mask!();
		let j = (next >> shift_per_level!()) & mask!();
		let i = (next >> 2 * shift_per_level!()) & mask!();

		let offset = offset!(id, self.slab_struct_size);

		unsafe {
			let mut lock = self.lock.read();

			if self.data.is_null() {
				lock.unlock();
				{
					let _lock = self.lock.write();
					if self.data.is_null() {
						self.data = map(1) as *mut *mut *mut *mut u8;
						if self.data.is_null() {
							return Err(ErrorKind::Alloc.into());
						}
					}
				}
				lock = self.lock.read();
			}

			if (*self.data.add(i)).is_null() {
				lock.unlock();
				{
					let _lock = self.lock.write();
					if (*self.data.add(i)).is_null() {
						*self.data.add(i) = map(1) as *mut *mut *mut u8;
						if (*self.data.add(i)).is_null() {
							return Err(ErrorKind::Alloc.into());
						}
					}
				}
				lock = self.lock.read();
			}

			if (*(*self.data.add(i)).add(j)).is_null() {
				lock.unlock();
				{
					let _lock = self.lock.write();
					if (*(*self.data.add(i)).add(j)).is_null() {
						*(*self.data.add(i)).add(j) = map(1) as *mut *mut u8;
						if (*(*self.data.add(i)).add(j)).is_null() {
							return Err(ErrorKind::Alloc.into());
						}
					}
				}
				lock = self.lock.read();
			}

			if (*(*(*self.data.add(i)).add(j)).add(k)).is_null() {
				lock.unlock();
				let _lock = self.lock.write();
				if (*(*(*self.data.add(i)).add(j)).add(k)).is_null() {
					*(*(*self.data.add(i)).add(j)).add(k) = map(1) as *mut u8;
					if (*(*(*self.data.add(i)).add(j)).add(k)).is_null() {
						return Err(ErrorKind::Alloc.into());
					}
				}
			}
		}

		let ret = Slab::new(
			unsafe { (*(*(*self.data.add(i)).add(j)).add(k)).add(offset) } as *mut Slab,
			self.slab_size,
			id,
		);
		Ok(ret)
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use core::mem::size_of;
	use core::slice::from_raw_parts_mut;
	use sys::getalloccount;

	#[test]
	fn test_slab1() {
		let initial = unsafe { getalloccount() };
		{
			let mut sa1 = SlabAllocator::new(224, 128, 256, 1).unwrap();
			let mut slab1 = sa1.alloc().unwrap();
			assert_eq!(slab1.id, 1);

			for i in 0..128 {
				slab1.get_mut()[i] = i as u8;
			}

			let mut slab2 = sa1.alloc().unwrap();
			assert_eq!(slab2.id, 2);
			for i in 0..128 {
				slab2.get_mut()[i] = (i + 1) as u8;
			}

			for i in 0..128 {
				assert_eq!(slab1.get()[i], i as u8);
			}
			for i in 0..128 {
				assert_eq!(slab2.get()[i], (i + 1) as u8);
			}

			sa1.cleanup();
		}
		assert_eq!(unsafe { getalloccount() }, initial);
	}

	#[test]
	fn test_slab_free() {
		let initial = unsafe { getalloccount() };
		{
			let mut sa1 = SlabAllocator::new(224, 128, 256, 1).unwrap();
			let mut slab1 = sa1.alloc().unwrap();
			assert_eq!(slab1.id, 1);

			sa1.free(&mut slab1);

			let mut slab2 = sa1.alloc().unwrap();
			assert_eq!(slab2.id, 0);

			let mut slab3 = sa1.alloc().unwrap();
			assert_eq!(slab3.id, 2);

			let mut slab4 = sa1.alloc().unwrap();
			assert_eq!(slab4.id, 3);
			let mut slab5 = sa1.alloc().unwrap();
			assert_eq!(slab5.id, 4);
			let slab6 = sa1.alloc().unwrap();
			assert_eq!(slab6.id, 5);

			sa1.free(&mut slab5);
			let slab7 = sa1.alloc().unwrap();
			assert_eq!(slab7.id, 1);

			let slab8 = sa1.alloc().unwrap();
			assert_eq!(slab8.id, 6);

			sa1.free(&mut slab4);
			sa1.free(&mut slab3);
			sa1.free(&mut slab2);

			let slab9 = sa1.alloc().unwrap();
			assert_eq!(slab9.id, 4);

			let slab10 = sa1.alloc().unwrap();
			assert_eq!(slab10.id, 3);

			let slab11 = sa1.alloc().unwrap();
			assert_eq!(slab11.id, 2);

			let slab12 = sa1.alloc().unwrap();
			assert_eq!(slab12.id, 7);

			let mut slab13 = sa1.alloc().unwrap();
			assert_eq!(slab13.id, 8);

			sa1.free(&mut slab13);
			//sa1.free(&mut slab13);

			sa1.cleanup();
		}
		assert_eq!(unsafe { getalloccount() }, initial);
	}

	const SIZE: usize = 32;
	const COUNT: usize = 1024 * 100;
	const ITER: usize = 10;

	#[test]
	fn test_slab2() {
		let initial = unsafe { getalloccount() };
		{
			let mut sa1 =
				SlabAllocator::new(SIZE, (COUNT + 10) as u64, (COUNT + 10) as u64, 20).unwrap();

			let pages_needed = 1 + divide_usize(COUNT * size_of::<Slab>(), page_size!());
			let slabs_ptr = unsafe { map(pages_needed) };
			let slabs = unsafe { from_raw_parts_mut(slabs_ptr as *mut Slab, COUNT) };

			let _start = getmicros!();
			for _ in 0..ITER {
				for i in 0..COUNT {
					if i % (1024 * 1024) == 0 {
						if i != 0 {
							print!("loop: ");
							print_num!(i);
							println!("");
						}
					}
					slabs[i] = sa1.alloc().unwrap();
					for j in 0..SIZE {
						slabs[i].get_mut()[j] = b'a' + ((i + j) % 26) as u8;
					}
				}

				for i in 0..COUNT {
					if i % (1024 * 1024) == 0 {
						if i != 0 {
							print!("free loop: ");
							print_num!(i);
							println!("");
						}
					}
					for j in 0..SIZE {
						assert_eq!(slabs[i].get()[j], b'a' + ((i + j) % 26) as u8);
					}
					sa1.free(&mut slabs[i]);
				}
			}

			/*
			print!("micros=");
			print_num!(getmicros!() - _start);
			println!("");
					*/

			assert_eq!(aload!(&sa1.total_slabs), (COUNT + 1) as u64);
			assert_eq!(sa1.free_slabs, (COUNT + 1) as u64);
			sa1.cleanup();

			unsafe {
				unmap(slabs_ptr, pages_needed);
			}
		}
		assert_eq!(unsafe { getalloccount() }, initial);
	}

	// test malloc/free for comparison
	extern "C" {
		pub fn malloc(size: usize) -> *mut u8;
		pub fn free(ptr: *mut u8);
	}

	#[test]
	fn test_malloc() {
		let pages_needed = 1 + divide_usize(COUNT * size_of::<Slab>(), page_size!());
		let slabs_ptr = unsafe { map(pages_needed) };
		let slabs = unsafe { from_raw_parts_mut(slabs_ptr as *mut Slab, COUNT) };

		let _start = getmicros!();
		for _ in 0..ITER {
			for i in 0..COUNT {
				if i % (1024 * 1024) == 0 {
					if i != 0 {
						print!("loop: ");
						print_num!(i);
						println!("");
					}
				}
				unsafe {
					slabs[i].data = malloc(SIZE);
				}
				slabs[i].len = SIZE;
				for j in 0..SIZE {
					slabs[i].get_mut()[j] = b'a' + ((i + j) % 26) as u8;
				}
			}

			for i in 0..COUNT {
				if i % (1024 * 1024) == 0 {
					if i != 0 {
						print!("free loop: ");
						print_num!(i);
						println!("");
					}
				}
				for j in 0..SIZE {
					assert_eq!(slabs[i].get()[j], b'a' + ((i + j) % 26) as u8);
				}
				unsafe {
					free(slabs[i].data);
				}
			}
		}

		/*
		print!("mallocmicros=");
		print_num!(getmicros!() - _start);
		println!("");
				*/

		unsafe {
			unmap(slabs_ptr, pages_needed);
		}
	}

	#[test]
	fn test_reconstruct() {
		let initial = unsafe { getalloccount() };
		{
			let mut sa1 = SlabAllocator::new(224, 128, 256, 1).unwrap();
			let slab1 = sa1.alloc().unwrap();
			assert_eq!(slab1.id, 1);
			let mut slab2 = Slab::from_raw(slab1.get_raw(), slab1.id);

			assert_eq!(aload!(&sa1.total_slabs), 2);
			assert_eq!(sa1.free_slabs, 1);

			sa1.free(&mut slab2);

			assert_eq!(aload!(&sa1.total_slabs), 2);
			assert_eq!(sa1.free_slabs, 2);

			sa1.cleanup();
		}
		assert_eq!(unsafe { getalloccount() }, initial);
	}
}
