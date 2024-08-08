use rand::{distributions::Alphanumeric, Rng};
use sqlx::Row;
use std::path::PathBuf;
use tracing::{trace, warn};
use copper_ds_core::{
	api::blob::{BlobHandle, BlobInfo, Blobstore, BlobstoreTmpWriter},
	errors::BlobstoreError,
};
use copper_util::mime::MimeType;

use super::LocalDataset;

impl Blobstore for LocalDataset {
	async fn new_blob(&self, mime: &MimeType) -> Result<BlobstoreTmpWriter, BlobstoreError> {
		let tmp_path = loop {
			let name: String = rand::thread_rng()
				.sample_iter(&Alphanumeric)
				.take(8)
				.map(char::from)
				.collect();

			let path = self.blobstore_tmp.join(name);
			if !path.exists() {
				break path;
			}
		};

		trace!(
			message = "Initialized a new blob",
			path = ?tmp_path,
			mime = ?mime,
		);

		Ok(BlobstoreTmpWriter::new(tmp_path, mime.clone())?)
	}

	async fn finish_blob(
		&self,
		mut blob: BlobstoreTmpWriter,
	) -> Result<BlobHandle, BlobstoreError> {
		trace!(
			message = "Finishing blob",
			path_to_tmp_blob = ?blob.path_to_file,
		);

		let final_path_rel = loop {
			let name: String = rand::thread_rng()
				.sample_iter(&Alphanumeric)
				.take(16)
				.map(char::from)
				.collect();

			let rel = format!("{name}{}", blob.mime.extension());
			let path = self.blobstore_root.join(&rel);
			if !path.exists() {
				break rel;
			}
		};

		std::fs::rename(
			&blob.path_to_file,
			self.blobstore_root.join(&final_path_rel),
		)?;

		let mut conn = self
			.pool
			.acquire()
			.await
			.map_err(|e| BlobstoreError::DbError(Box::new(e)))?;

		let res = sqlx::query("INSERT INTO meta_blobs (data_type, file_path) VALUES (?, ?);")
			.bind(blob.mime.to_string())
			.bind(final_path_rel)
			.execute(&mut *conn)
			.await;

		let id = match res {
			Err(e) => return Err(BlobstoreError::DbError(Box::new(e))),
			Ok(res) => res.last_insert_rowid(),
		};

		// Make sure file isn't deleted when blob is dropped
		blob.is_finished = true;

		trace!(
			message = "Finished blob",
			path_to_tmp_blob = ?blob.path_to_file,
			blob_handle = id
		);

		return Ok(BlobHandle::from(u32::try_from(id).unwrap()));
	}

	async fn delete_blob(&self, blob: BlobHandle) -> Result<(), BlobstoreError> {
		trace!(
			message = "Deleting blob",
			blob_handle = ?blob,
		);

		let mut conn = self
			.pool
			.acquire()
			.await
			.map_err(|e| BlobstoreError::DbError(Box::new(e)))?;

		// We intentionally don't use a transaction here.
		// A blob shouldn't point to a partially-deleted file.

		let res = sqlx::query("SELECT file_path FROM meta_blobs WHERE id=?;")
			.bind(u32::from(blob))
			.fetch_one(&mut *conn)
			.await;

		let rel_file_path = match res {
			Err(sqlx::Error::RowNotFound) => return Err(BlobstoreError::InvalidBlobHandle),
			Err(e) => return Err(BlobstoreError::DbError(Box::new(e))),
			Ok(res) => PathBuf::from(res.get::<&str, _>("file_path")),
		};
		let file_path = self.blobstore_root.join(&rel_file_path);

		// Delete blob metadata
		if let Err(e) = sqlx::query("DELETE FROM meta_blobs WHERE id=?;")
			.bind(u32::from(blob))
			.execute(&mut *conn)
			.await
		{
			return Err(BlobstoreError::DbError(Box::new(e)));
		};

		if !file_path.exists() {
			warn!(
				message = "Trying to delete blob, but it doesn't exist. Skipping.",
				blob_handle = ?blob,
				blob_path = ?file_path
			);
		} else {
			// Delete blob file
			std::fs::remove_file(file_path)?;
		}

		trace!(
			message = "Deleted blob",
			blob_handle = ?blob,
		);

		return Ok(());
	}

	async fn all_blobs(&self) -> Result<Vec<BlobHandle>, BlobstoreError> {
		let mut conn = self
			.pool
			.acquire()
			.await
			.map_err(|e| BlobstoreError::DbError(Box::new(e)))?;

		let res = sqlx::query("SELECT id FROM meta_blobs ORDER BY id;")
			.fetch_all(&mut *conn)
			.await
			.map_err(|e| BlobstoreError::DbError(Box::new(e)))?;

		Ok(res
			.into_iter()
			.map(|x| x.get::<u32, _>("id").into())
			.collect())
	}

	async fn blob_size(&self, blob: BlobHandle) -> Result<u64, BlobstoreError> {
		let mut conn = self
			.pool
			.acquire()
			.await
			.map_err(|e| BlobstoreError::DbError(Box::new(e)))?;

		let res = sqlx::query("SELECT file_path FROM meta_blobs WHERE id=?;")
			.bind(u32::from(blob))
			.fetch_one(&mut *conn)
			.await;

		let rel_file_path = match res {
			Err(sqlx::Error::RowNotFound) => return Err(BlobstoreError::InvalidBlobHandle),
			Err(e) => return Err(BlobstoreError::DbError(Box::new(e))),
			Ok(res) => PathBuf::from(res.get::<&str, _>("file_path")),
		};
		let file_path = self.blobstore_root.join(rel_file_path);

		let meta = std::fs::metadata(file_path)?;

		return Ok(meta.len());
	}

	async fn get_blob(&self, blob: BlobHandle) -> Result<BlobInfo, BlobstoreError> {
		let mut conn = self
			.pool
			.acquire()
			.await
			.map_err(|e| BlobstoreError::DbError(Box::new(e)))?;

		let res = sqlx::query("SELECT file_path, data_type FROM meta_blobs WHERE id=?;")
			.bind(u32::from(blob))
			.fetch_one(&mut *conn)
			.await;

		let (rel_file_path, data_type) = match res {
			Err(sqlx::Error::RowNotFound) => return Err(BlobstoreError::InvalidBlobHandle),
			Err(e) => return Err(BlobstoreError::DbError(Box::new(e))),
			Ok(res) => (
				PathBuf::from(res.get::<&str, _>("file_path")),
				res.get::<&str, _>("data_type").into(),
			),
		};
		let file_path = self.blobstore_root.join(&rel_file_path);
		let file = tokio::fs::File::open(&file_path).await?;

		return Ok(BlobInfo {
			handle: blob,
			mime: data_type,
			data: Box::pin(file),
		});
	}
}
