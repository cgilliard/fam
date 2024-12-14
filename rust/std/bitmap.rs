use err;
use std::blob::Blob;
use std::error::Error;
use std::error::ErrorKind::Alloc;
use std::result::{Result, Result::Err, Result::Ok};
use std::util::copy_slice;

pub struct BitMap {
	blob: Blob,
}

impl BitMap {
	pub fn new() -> Self {
		// unwrap ok because size is 0 so no failures
		let blob = Blob::new(0).unwrap();
		BitMap { blob }
	}

	pub fn allocate(&mut self) -> Result<&mut [u8], Error> {
		let slice = self.blob.get_mut(0, 100).unwrap();
		let slice2 = self.blob.get_mut(100, 200).unwrap();
		slice[0] = 1;
		slice2[0] = 2;
		Err(err!(Alloc))
	}

	pub fn free(&mut self) {}

	fn _resize(&mut self, pages: usize) -> Result<(), Error> {
		match Blob::new(pages) {
			Ok(blob) => {
				let cur_pages = self.blob.pages();
				let copy_pages = if cur_pages > pages { pages } else { cur_pages };
				let len = copy_pages * page_size!();
				// unwrap ok because len <= pages * page_size!()
				let b0 = blob.get_mut(0, len).unwrap();
				let b1 = self.blob.get(0, len).unwrap();
				copy_slice(b1, b0, len);
				self.blob = blob;
				Ok(())
			}
			Err(e) => Err(e),
		}
	}
}
