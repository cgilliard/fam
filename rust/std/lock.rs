use core::cell::UnsafeCell;
use prelude::*;

const WFLAG: u64 = 0x1u64 << 63u64;
const WREQUEST: u64 = 0x1u64 << 62u64;

#[macro_export]
macro_rules! lock {
	() => {{
		use core::cell::UnsafeCell;
		Lock {
			state: UnsafeCell::new(0),
		}
	}};
}

#[macro_export]
macro_rules! lock_box {
	() => {{
		LockBox::new()
	}};
}

pub struct Lock {
	pub state: UnsafeCell<u64>,
}

struct LockBoxInner {
	lock: Lock,
}

pub struct LockBox {
	inner: Rc<LockBoxInner>,
}

pub struct LockReadGuard<'a> {
	lock: &'a Lock,
	unlock: bool,
}

pub struct LockWriteGuard<'a> {
	lock: &'a Lock,
	unlock: bool,
}

impl LockWriteGuard<'_> {
	pub fn unlock(&mut self) {
		if !self.unlock {
			let state = unsafe { &mut *self.lock.state.get() };
			astore!(&mut *state, 0);
			self.unlock = true;
		}
	}
}

impl LockReadGuard<'_> {
	pub fn unlock(&mut self) {
		if !self.unlock {
			let state = unsafe { &mut *self.lock.state.get() };
			asub!(&mut *state, 1);
			self.unlock = true;
		}
	}
}

impl Drop for LockWriteGuard<'_> {
	fn drop(&mut self) {
		self.unlock();
	}
}

impl Drop for LockReadGuard<'_> {
	fn drop(&mut self) {
		self.unlock();
	}
}

impl Lock {
	pub fn new() -> Self {
		Self {
			state: 0_u64.into(),
		}
	}

	pub fn read<'a>(&'a self) -> LockReadGuard<'a> {
		let state = unsafe { &mut *self.state.get() };
		loop {
			let x = aload!(state) & !(WFLAG | WREQUEST);
			let y = x + 1;
			if cas!(state, &x, y) {
				break;
			}
			sched_yield!();
		}
		LockReadGuard {
			lock: self,
			unlock: false,
		}
	}

	pub fn write<'a>(&'a self) -> LockWriteGuard<'a> {
		let state = unsafe { &mut *self.state.get() };

		loop {
			let x = aload!(state) & !(WFLAG | WREQUEST);
			if cas!(state, &x, WREQUEST) {
				break;
			}
			sched_yield!();
		}
		loop {
			let x = WREQUEST;
			if cas!(state, &x, WFLAG) {
				break;
			}
			sched_yield!();
		}
		LockWriteGuard {
			lock: self,
			unlock: false,
		}
	}
}

impl Clone for LockBox {
	fn clone(&self) -> Result<Self, Error> {
		match self.inner.clone() {
			Ok(inner) => Ok(Self { inner }),
			Err(e) => Err(e),
		}
	}
}

impl LockBox {
	pub fn new() -> Result<Self, Error> {
		match Rc::new(LockBoxInner { lock: lock!() }) {
			Ok(inner) => Ok(Self { inner }),
			Err(e) => Err(e),
		}
	}

	pub fn read<'a>(&'a self) -> LockReadGuard<'a> {
		self.inner.lock.read()
	}

	pub fn write<'a>(&'a self) -> LockWriteGuard<'a> {
		self.inner.lock.write()
	}
}

#[cfg(test)]
mod test {
	use super::WFLAG;
	use prelude::*;
	use std::lock::Lock;
	#[test]
	fn test_lock() {
		let x = lock!();
		assert_eq!(unsafe { *x.state.get() }, 0);
		{
			let _v = x.write();
			assert_eq!(unsafe { *x.state.get() }, WFLAG);
		}
		assert_eq!(unsafe { *x.state.get() }, 0);
		{
			let _v = x.write();
			assert_eq!(unsafe { *x.state.get() }, WFLAG);
		}
		{
			let _v = x.read();
			assert_eq!(unsafe { *x.state.get() }, 1);
			{
				let _v = x.read();
				assert_eq!(unsafe { *x.state.get() }, 2);
				{
					let _v = x.read();
					assert_eq!(unsafe { *x.state.get() }, 3);
				}
				assert_eq!(unsafe { *x.state.get() }, 2);
			}
			assert_eq!(unsafe { *x.state.get() }, 1);
		}
		assert_eq!(unsafe { *x.state.get() }, 0);
	}

	#[test]
	fn test_lock_box() {
		let x = lock_box!().unwrap();
		let y = x.clone().unwrap();
		assert_eq!(unsafe { *x.inner.lock.state.get() }, 0);
		{
			let _v = x.write();
			assert_eq!(unsafe { *x.inner.lock.state.get() }, WFLAG);
		}
		assert_eq!(unsafe { *x.inner.lock.state.get() }, 0);
		{
			let _v = x.write();
			assert_eq!(unsafe { *y.inner.lock.state.get() }, WFLAG);
		}
		{
			let _v = x.read();
			assert_eq!(unsafe { *x.inner.lock.state.get() }, 1);
			{
				let _v = x.read();
				assert_eq!(unsafe { *x.inner.lock.state.get() }, 2);
				{
					let _v = y.read();
					assert_eq!(unsafe { *x.inner.lock.state.get() }, 3);
				}
				assert_eq!(unsafe { *y.inner.lock.state.get() }, 2);
			}
			assert_eq!(unsafe { *x.inner.lock.state.get() }, 1);
		}
		assert_eq!(unsafe { *y.inner.lock.state.get() }, 0);
	}
}
