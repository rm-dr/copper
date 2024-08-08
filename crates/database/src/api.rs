use ufo_util::mime::MimeType;

use crate::{
	blobstore::api::{BlobHandle, BlobstoreTmpWriter},
	metastore::api::Metastore,
};

pub trait UFODatabase
where
	Self: Send + Sync,
{
	fn get_metastore(&mut self) -> &mut dyn Metastore;
	fn new_blob(&mut self, mime: &MimeType) -> BlobstoreTmpWriter;
	fn finish_blob(&mut self, blob: BlobstoreTmpWriter) -> BlobHandle;
}
