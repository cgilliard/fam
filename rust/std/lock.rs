use core::cell::UnsafeCell;
use core::convert::Into;
use core::ops::Drop;

const WFLAG: u64 = 0x1u64 << 63u64;

pub struct Lock {
	state: UnsafeCell<u64>,
}

pub struct LockReadGuard<'a> {
	lock: &'a Lock,
}

pub struct LockWriteGuard<'a> {
	lock: &'a Lock,
}

impl Drop for LockWriteGuard<'_> {
	fn drop(&mut self) {
		let state = unsafe { &mut *self.lock.state.get() };
		astore!(&mut *state, 0);
	}
}

impl Drop for LockReadGuard<'_> {
	fn drop(&mut self) {
		let state = unsafe { &mut *self.lock.state.get() };
		asub!(&mut *state, 1);
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
			let x = aload!(state) & !WFLAG;
			let y = x + 1;
			if cas!(state, &x, y) {
				break;
			}
			sched_yield!();
		}
		LockReadGuard { lock: self }
	}

	pub fn write<'a>(&'a self) -> LockWriteGuard<'a> {
		let state = unsafe { &mut *self.state.get() };
		loop {
			let x = 0;
			if cas!(state, &x, WFLAG) {
				break;
			}
			sched_yield!();
		}
		LockWriteGuard { lock: self }
	}
}

#[cfg(test)]
mod test {
	use super::WFLAG;
	use std::lock::Lock;
	#[test]
	fn test_lock() {
		let x = Lock::new();
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
}
