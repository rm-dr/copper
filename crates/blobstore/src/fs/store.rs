use std::{fs::File, io::Write, path::PathBuf};

use smartstring::{LazyCompact, SmartString};
use ufo_util::mime::MimeType;

use crate::api::{BlobHandle, BlobStore};

#[derive(Debug, Clone)]
pub struct FsBlobHandle {
	name: SmartString<LazyCompact>,
	mime: MimeType,
}

impl BlobHandle for FsBlobHandle {
	fn to_db_str(&self) -> String {
		self.name.to_string()
	}

	fn from_db_str(s: &str) -> Self {
		Self {
			name: s.into(),
			mime: MimeType::Blob,
		}
	}

	fn get_type(&self) -> &MimeType {
		&self.mime
	}
}

pub struct FsBlobWriter {
	file: Option<File>,
	handle: FsBlobHandle,

	// Used for cleanup
	path: PathBuf,
	is_finished: bool,
}

impl Write for FsBlobWriter {
	fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
		self.file.as_mut().unwrap().write(buf)
	}

	fn flush(&mut self) -> std::io::Result<()> {
		self.file.as_mut().unwrap().flush()
	}
}

// TODO: tmp dir
// TODO: test this
impl Drop for FsBlobWriter {
	fn drop(&mut self) {
		// If we never finish a writer, drop the file.
		self.file.take();
		std::fs::remove_file(&self.path).unwrap();
	}
}

pub struct FsBlobStore {
	root: PathBuf,
	idx: usize,
}

unsafe impl Send for FsBlobStore {}

impl FsBlobStore {
	pub fn open(root: PathBuf) -> Self {
		Self { root, idx: 0 }
	}
}

impl BlobStore for FsBlobStore {
	type Handle = FsBlobHandle;
	type Writer = FsBlobWriter;

	fn new_blob(&mut self, mime: &MimeType) -> Self::Writer {
		let i = self.idx;
		self.idx += 1;

		let p = self.root.join(format!("{i}{}", mime.extension()));
		let f = File::create(&p).unwrap();
		FsBlobWriter {
			path: p,
			file: Some(f),
			handle: FsBlobHandle {
				name: format!("{i}").into(),
				mime: mime.clone(),
			},
			is_finished: false,
		}
	}

	fn finish_blob(&mut self, mut blob: Self::Writer) -> FsBlobHandle {
		blob.is_finished = true;
		blob.handle.clone()
	}
}
