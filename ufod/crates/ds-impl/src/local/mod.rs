use std::{
	path::{Path, PathBuf},
	sync::Mutex,
};

use futures::executor::block_on;
use sqlx::{Connection, Row, SqliteConnection};
use ufo_ds_core::{api::Dataset, errors::MetastoreError};
use ufo_pipeline::api::PipelineNodeStub;

mod blob;
mod meta;
mod pipe;

pub struct LocalDataset {
	/// Database connection
	conn: Mutex<SqliteConnection>,

	// Blobstore
	blobstore_root: PathBuf,
	blobstore_tmp: PathBuf,
}

impl<PipelineNodeStubType: PipelineNodeStub> Dataset<PipelineNodeStubType> for LocalDataset {}

impl LocalDataset {
	/// Create a new [`LocalDataset`].
	/// `db_root` must not exist and be empty.
	pub fn create(db_root: &Path) -> Result<(), MetastoreError> {
		// Init root dir
		if db_root.exists() {
			panic!("TODO: proper error")
		} else {
			std::fs::create_dir(db_root).unwrap();
		}

		// Constant configs
		let blob_storage_dir = "blobs";
		let db_name = "dataset.sqlite";

		// Make database
		let db_file = db_root.join(db_name);
		let db_addr = format!("sqlite:{}?mode=rwc", db_file.to_str().unwrap());
		let mut conn = block_on(SqliteConnection::connect(&db_addr))
			.map_err(|e| MetastoreError::DbError(Box::new(e)))?;
		block_on(sqlx::query(include_str!("./init.sql")).execute(&mut conn)).unwrap();
		block_on(
			sqlx::query("INSERT INTO meta_meta (var, val) VALUES (?, ?);")
				.bind("ufo_version")
				.bind(env!("CARGO_PKG_VERSION"))
				.execute(&mut conn),
		)
		.unwrap();

		// Initialize blob store
		let blob_storage_dir_absolute = db_root.join(blob_storage_dir);
		std::fs::create_dir(blob_storage_dir_absolute).unwrap();
		block_on(
			sqlx::query("INSERT INTO meta_meta (var, val) VALUES (?,?);")
				.bind("blob_dir")
				.bind(blob_storage_dir)
				.execute(&mut conn),
		)
		.unwrap();

		Ok(())
	}

	pub fn open(db_root: &Path) -> Result<Self, ()> {
		let db_file = db_root.join("dataset.sqlite");
		let db_addr = format!("sqlite:{}?mode=rw", db_file.to_str().unwrap());
		let mut conn = block_on(SqliteConnection::connect(&db_addr)).unwrap();

		// TODO: check version, blobstore dir

		let blob_dir = db_root.join({
			let res = block_on(
				sqlx::query("SELECT val FROM meta_meta WHERE var=\"blob_dir\";")
					.fetch_one(&mut conn),
			)
			.unwrap();
			res.get::<String, _>("val")
		});

		Ok(Self {
			blobstore_root: blob_dir.clone(),
			// TODO: configurable & clear
			blobstore_tmp: blob_dir.join("tmp"),
			conn: Mutex::new(conn),
		})
	}
}
