use std::{
	path::{Path, PathBuf},
	sync::Mutex,
};

use futures::executor::block_on;
use sqlx::{Connection, Row, SqliteConnection};
use ufo_util::mime::MimeType;

use crate::blobstore::api::{BlobHandle, BlobstoreTmpWriter};

use super::super::api::Blobstore;

pub struct FsBlobstore {
	root: PathBuf,
	idx: Mutex<u32>,
	conn: Mutex<SqliteConnection>,
}

unsafe impl Send for FsBlobstore {}

impl FsBlobstore {
	pub(crate) fn create(
		root_dir: &Path,
		blob_db_file: &Path,
		blob_storage_dir: &Path,
	) -> Result<(), ()> {
		let blob_db_absolute = root_dir.join(&blob_db_file);
		let blob_storage_dir_absolute = root_dir.join(&blob_storage_dir);

		if blob_storage_dir.exists() {
			return Err(());
		}
		std::fs::create_dir(blob_storage_dir_absolute).unwrap();
		let db_addr = format!("sqlite:{}?mode=rwc", blob_db_absolute.to_str().unwrap());
		let mut conn = block_on(SqliteConnection::connect(&db_addr)).unwrap();

		block_on(sqlx::query(include_str!("./init.sql")).execute(&mut conn)).unwrap();
		block_on(
			sqlx::query("INSERT INTO meta (var, val) VALUES (?,?), (?,?), (?,?);")
				.bind("ufo_version")
				.bind(env!("CARGO_PKG_VERSION"))
				.bind("idx_counter")
				.bind(0)
				.bind("blob_dir")
				.bind(blob_storage_dir.to_str().unwrap())
				.execute(&mut conn),
		)
		.unwrap();

		Ok(())
	}

	pub(crate) fn open(db_root_dir: &Path, blob_db_name: &str) -> Result<Self, ()> {
		let database = db_root_dir.join(blob_db_name);
		let db_addr = format!("sqlite:{}?mode=rwc", database.to_str().unwrap());
		let mut conn = block_on(SqliteConnection::connect(&db_addr)).unwrap();

		let idx_counter: u32 = {
			let res = block_on(
				sqlx::query("SELECT val FROM meta WHERE var=\"idx_counter\";").fetch_one(&mut conn),
			)
			.unwrap();
			res.get::<String, _>("val").parse().unwrap()
		};

		let blob_dir = db_root_dir.join({
			let res = block_on(
				sqlx::query("SELECT val FROM meta WHERE var=\"blob_dir\";").fetch_one(&mut conn),
			)
			.unwrap();
			res.get::<String, _>("val")
		});

		Ok(Self {
			root: blob_dir,
			idx: Mutex::new(idx_counter),
			conn: Mutex::new(conn),
		})
	}
}

impl Blobstore for FsBlobstore {
	fn new_blob(&mut self, mime: &MimeType) -> BlobstoreTmpWriter {
		let mut li = self.idx.lock().unwrap();
		let i = *li;
		*li += 1;

		block_on(
			sqlx::query("UPDATE meta SET val=? WHERE var=\"idx_counter\";")
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

	fn finish_blob(&mut self, mut blob: BlobstoreTmpWriter) -> BlobHandle {
		block_on(
			sqlx::query("INSERT INTO blobs (data_type, file_path) VALUES (?, ?);")
				.bind(blob.handle.get_type().to_db_str())
				.bind(blob.path_to_file.to_str().unwrap())
				.execute(&mut *self.conn.lock().unwrap()),
		)
		.unwrap();

		blob.is_finished = true;
		blob.handle.clone()
	}
}
