use std::{
	io::{Read, Seek, SeekFrom},
	sync::Arc,
};

/// Read helper for `Blob` channels.
/// Stores a vec of `Blob` fragments and provides a `Read`
/// over a virtual, concatenated Vec<u8>
///
/// The sum of the sizes of all fragments must fit inside
/// a u64. Things will break if they don't. (TODO: fix?)
pub struct ArcVecBuffer {
	// Note the Arc. This minimizes memory use,
	// since each fragment is allocated exactly once!
	buffer: Vec<Arc<Vec<u8>>>,
	cursor: u64,
}

impl ArcVecBuffer {
	pub fn new() -> Self {
		Self {
			buffer: Vec::new(),
			cursor: 0,
		}
	}

	pub fn push_back(&mut self, data: Arc<Vec<u8>>) {
		self.buffer.push(data);
	}

	pub fn len(&self) -> u64 {
		let mut l = 0u64;
		for b in &self.buffer {
			l += u64::try_from(b.len()).unwrap()
		}
		l
	}

	/// Clear this buffer.
	pub fn clear(&mut self) {
		self.buffer.clear();
		self.cursor = 0;
	}
}

impl Read for ArcVecBuffer {
	fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
		if self.buffer.is_empty() {
			return Ok(0);
		}

		let mut written: usize = 0;
		let mut space_left: usize = buf.len();
		loop {
			// outer idx: position in self.buffer
			// inner idx: position inside that buffer
			let (inner_idx, outer_idx) = {
				let mut i = 0;
				let mut c = self.cursor;
				loop {
					let l = u64::try_from(self.buffer[i].len()).unwrap();
					if c < l {
						break (usize::try_from(c).unwrap(), i);
					}

					i += 1;
					c -= l;

					if i >= self.buffer.len() {
						break (
							self.buffer.last().as_ref().unwrap().len(),
							self.buffer.len() - 1,
						);
					}
				}
			};

			let current = &self.buffer[outer_idx];
			let len_current = current.len();
			let current_left = len_current - inner_idx;
			let to_write = current_left.min(space_left);

			if to_write == 0 {
				return Ok(written);
			}

			buf[written..written + to_write]
				.copy_from_slice(&current[inner_idx..inner_idx + to_write]);

			self.cursor += u64::try_from(to_write).unwrap();
			written += to_write;
			space_left -= to_write;

			if space_left == 0 {
				return Ok(written);
			}
		}
	}
}

impl Seek for ArcVecBuffer {
	fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
		match pos {
			SeekFrom::Current(x) => {
				if 0 <= x {
					let x = u64::try_from(x).unwrap();
					self.cursor += x;
					if self.len() < self.cursor {
						self.cursor = self.len()
					}
				} else {
					let x = u64::try_from(x.abs()).unwrap();
					if self.cursor < x {
						return Err(std::io::ErrorKind::InvalidInput.into());
					}
					self.cursor -= x;
				}
				return Ok(self.cursor);
			}
			SeekFrom::End(x) => {
				if 0 <= x {
					self.cursor = self.len()
				} else {
					let x = u64::try_from(x.abs()).unwrap();
					if self.len() < x {
						return Err(std::io::ErrorKind::InvalidInput.into());
					}
					self.cursor = self.len() - x;
				}
				return Ok(self.cursor);
			}
			SeekFrom::Start(x) => {
				self.cursor = x;
				if self.cursor > self.len() {
					self.cursor = self.len()
				}
				return Ok(self.cursor);
			}
		}
	}
}
