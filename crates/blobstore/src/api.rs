use std::{fs::File, io::Write};

use smartstring::{LazyCompact, SmartString};
use ufo_util::mime::MimeType;

#[derive(Debug, Clone)]
pub struct BlobHandle {
	pub(crate) name: SmartString<LazyCompact>,
}

impl BlobHandle {
	pub fn to_db_str(&self) -> String {
		self.name.clone().into()
	}

	pub fn from_db_str(s: &str) -> Self {
		Self { name: s.into() }
	}
}

pub struct BlobWriter {
	pub(crate) file: File,
	pub(crate) handle: BlobHandle,
}

impl Write for BlobWriter {
	fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
		self.file.write(buf)
	}

	fn flush(&mut self) -> std::io::Result<()> {
		self.file.flush()
	}
}

pub trait BlobStore {
	fn new_blob(&mut self, mime: &MimeType) -> BlobWriter;
	fn finish_blob(&mut self, blob: BlobWriter) -> Result<BlobHandle, ()>;
}
