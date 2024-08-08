use smartstring::{LazyCompact, SmartString};
use std::{fs::File, io::Write, path::PathBuf};
use ufo_util::mime::MimeType;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlobHandle {
	name: SmartString<LazyCompact>,
	mime: MimeType,
}

impl BlobHandle {
	pub fn new(name: &str, mime: &MimeType) -> Self {
		Self {
			name: name.into(),
			mime: mime.clone(),
		}
	}

	pub fn to_db_str(&self) -> String {
		self.name.to_string()
	}

	pub fn from_db_str(s: &str) -> Self {
		Self {
			name: s.into(),
			mime: MimeType::Blob,
		}
	}

	pub fn get_type(&self) -> &MimeType {
		&self.mime
	}
}

pub struct BlobstoreTmpWriter {
	file: Option<File>,

	pub(crate) handle: BlobHandle,

	// Absolute path to blob store
	pub(crate) blob_store_root: PathBuf,

	// Path to this file, relative to blob_store_root
	pub(crate) path_to_file: PathBuf,

	// Used for cleanup
	pub(crate) is_finished: bool,
}

impl BlobstoreTmpWriter {
	pub(crate) fn new(blob_store_root: PathBuf, path_to_file: PathBuf, handle: BlobHandle) -> Self {
		let file = File::create(&path_to_file).unwrap();

		Self {
			file: Some(file),
			blob_store_root,
			handle,
			path_to_file,
			is_finished: false,
		}
	}
}

impl Write for BlobstoreTmpWriter {
	fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
		self.file.as_mut().unwrap().write(buf)
	}

	fn flush(&mut self) -> std::io::Result<()> {
		self.file.as_mut().unwrap().flush()
	}
}

impl Drop for BlobstoreTmpWriter {
	fn drop(&mut self) {
		self.file.take().unwrap().flush().unwrap();

		// If we never finished this writer, delete the file.
		if !self.is_finished {
			std::fs::remove_file(self.blob_store_root.join(&self.path_to_file)).unwrap();
		}
	}
}

pub trait Blobstore
where
	Self: Send + Sized,
{
	fn new_blob(&mut self, mime: &MimeType) -> BlobstoreTmpWriter;
	fn finish_blob(&mut self, blob: BlobstoreTmpWriter) -> BlobHandle;
}
