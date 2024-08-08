use futures::executor::block_on;
use std::path::PathBuf;
use ufo_ds_core::api::blob::{BlobHandle, Blobstore, BlobstoreTmpWriter};
use ufo_util::mime::MimeType;

use super::LocalDataset;

impl Blobstore for LocalDataset {
	fn new_blob(&self, mime: &MimeType) -> BlobstoreTmpWriter {
		let mut li = self.idx.lock().unwrap();
		let i = *li;
		*li += 1;

		block_on(
			sqlx::query("UPDATE meta_meta SET val=? WHERE var=\"idx_counter\";")
				.bind(*li)
				.execute(&mut *self.conn.lock().unwrap()),
		)
		.unwrap();

		let relative_path: PathBuf = format!("{i}{}", mime.extension()).into();
		let name = format!("{i}");

		BlobstoreTmpWriter::new(
			self.root.clone(),
			self.root.join(relative_path),
			BlobHandle::new(&name, mime),
		)
	}

	fn finish_blob(&self, mut blob: BlobstoreTmpWriter) -> BlobHandle {
		block_on(
			sqlx::query("INSERT INTO meta_blobs (data_type, file_path) VALUES (?, ?);")
				.bind(blob.handle.get_type().to_string())
				.bind(blob.path_to_file.to_str().unwrap())
				.execute(&mut *self.conn.lock().unwrap()),
		)
		.unwrap();

		blob.is_finished = true;
		blob.handle.clone()
	}
}
