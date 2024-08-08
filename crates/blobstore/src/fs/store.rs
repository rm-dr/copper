use std::{fs::File, path::PathBuf};

use ufo_util::mime::MimeType;

use crate::api::{BlobHandle, BlobStore, BlobWriter};

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
	fn new_blob(&mut self, mime: &MimeType) -> BlobWriter {
		let i = self.idx;
		self.idx += 1;

		let f = File::create(self.root.join(format!("{i}{}", mime.extension()))).unwrap();
		BlobWriter {
			file: f,
			handle: BlobHandle {
				name: format!("{i}").into(),
			},
		}
	}

	fn finish_blob(&mut self, blob: BlobWriter) -> Result<BlobHandle, ()> {
		Ok(blob.handle.clone())
	}
}
