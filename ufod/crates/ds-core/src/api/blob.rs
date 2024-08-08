use serde::{Deserialize, Serialize};
use std::{fs::File, io::Write, path::PathBuf, pin::Pin};
use tokio::io::AsyncRead;
use copper_util::mime::MimeType;
use utoipa::ToSchema;

use crate::errors::BlobstoreError;

#[derive(
	Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, ToSchema,
)]
#[serde(transparent)]
pub struct BlobHandle {
	id: u32,
}

impl From<BlobHandle> for u32 {
	fn from(value: BlobHandle) -> Self {
		value.id
	}
}

impl From<u32> for BlobHandle {
	fn from(value: u32) -> Self {
		Self { id: value }
	}
}

pub struct BlobstoreTmpWriter {
	file: Option<File>,

	pub mime: MimeType,

	// Path to this file
	pub path_to_file: PathBuf,

	// Used for cleanup
	pub is_finished: bool,
}

impl BlobstoreTmpWriter {
	pub fn new(path_to_file: PathBuf, mime: MimeType) -> Result<Self, std::io::Error> {
		let file = File::create(&path_to_file)?;

		Ok(Self {
			file: Some(file),
			mime,
			path_to_file,
			is_finished: false,
		})
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
			std::fs::remove_file(&self.path_to_file).unwrap();
		}
	}
}

pub struct BlobInfo {
	pub handle: BlobHandle,
	pub mime: MimeType,
	pub data: Pin<Box<dyn AsyncRead + Send>>,
}

#[allow(async_fn_in_trait)]
pub trait Blobstore
where
	Self: Send + Sync,
{
	async fn new_blob(&self, mime: &MimeType) -> Result<BlobstoreTmpWriter, BlobstoreError>;
	async fn finish_blob(&self, blob: BlobstoreTmpWriter) -> Result<BlobHandle, BlobstoreError>;
	async fn delete_blob(&self, blob: BlobHandle) -> Result<(), BlobstoreError>;
	async fn get_blob(&self, blob: BlobHandle) -> Result<BlobInfo, BlobstoreError>;
	async fn all_blobs(&self) -> Result<Vec<BlobHandle>, BlobstoreError>;
	async fn blob_size(&self, blob: BlobHandle) -> Result<u64, BlobstoreError>;
}
