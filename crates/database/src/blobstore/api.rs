use std::{io::Write, path::Path};
use ufo_util::mime::MimeType;

pub trait BlobHandle {
	fn to_db_str(&self) -> String;
	fn from_db_str(s: &str) -> Self;
	fn get_type(&self) -> &MimeType;
}

pub trait BlobStore
where
	Self: Send + Sized,
{
	type Handle: BlobHandle;
	type Writer: Write;

	fn open(path: &Path) -> Result<Self, ()>;
	fn create(path: &Path) -> Result<(), ()>;

	fn new_blob(&mut self, mime: &MimeType) -> Self::Writer;
	fn finish_blob(&mut self, blob: Self::Writer) -> Self::Handle;
}
