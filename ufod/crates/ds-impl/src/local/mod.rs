use std::{
	path::{Path, PathBuf},
	sync::Mutex,
};

use futures::executor::block_on;
use sqlx::{Connection, Row, SqliteConnection};
use tracing::{debug, error, info};
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
	pub fn create(ds_root: &Path) -> Result<(), MetastoreError> {
		// Init root dir
		if ds_root.exists() {
			panic!("TODO: proper error")
		} else {
			std::fs::create_dir(ds_root).unwrap();
		}

		// Constant configs
		let blob_storage_dir = "blobs";
		let blob_tmp_dir = ".blobs.tmp";
		let db_name = "dataset.sqlite";

		// Make database
		let db_file = ds_root.join(db_name);
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
		let blob_storage_dir_absolute = ds_root.join(blob_storage_dir);
		let blob_tmp_dir_absolute = ds_root.join(blob_tmp_dir);
		std::fs::create_dir(blob_storage_dir_absolute).unwrap();
		std::fs::create_dir(blob_tmp_dir_absolute).unwrap();
		block_on(
			sqlx::query("INSERT INTO meta_meta (var, val) VALUES (?,?), (?,?);")
				.bind("blob_storage_dir")
				.bind(blob_storage_dir)
				.bind("blob_tmp_dir")
				.bind(blob_tmp_dir)
				.execute(&mut conn),
		)
		.unwrap();

		Ok(())
	}

	pub fn open(ds_root: &Path) -> Result<Self, ()> {
		debug!(
			message = "Opening dataset",
			ds_root = ?ds_root
		);

		let db_file = ds_root.join("dataset.sqlite");
		let db_addr = format!("sqlite:{}?mode=rw", db_file.to_str().unwrap());
		let mut conn = block_on(SqliteConnection::connect(&db_addr)).unwrap();

		// TODO: check version, blobstore dir

		let blob_storage_dir = ds_root.join({
			let res = block_on(
				sqlx::query("SELECT val FROM meta_meta WHERE var=\"blob_storage_dir\";")
					.fetch_one(&mut conn),
			)
			.unwrap();
			res.get::<String, _>("val")
		});

		let blob_tmp_dir = ds_root.join({
			let res = block_on(
				sqlx::query("SELECT val FROM meta_meta WHERE var=\"blob_tmp_dir\";")
					.fetch_one(&mut conn),
			)
			.unwrap();
			res.get::<String, _>("val")
		});

		let blob_tmp_dir_absolute = ds_root.join(&blob_tmp_dir);
		if blob_tmp_dir_absolute.exists() {
			if blob_tmp_dir_absolute.is_dir() {
				info!(
					message = "Deleting old blob tmp dir",
					directory = ?blob_tmp_dir_absolute
				);
				std::fs::remove_dir_all(&blob_tmp_dir_absolute).unwrap();
				std::fs::create_dir(blob_tmp_dir_absolute).unwrap();
			} else {
				error!(
					message = "Blobstore tmp dir is not a directory!",
					blob_tmp_dir = ?blob_tmp_dir_absolute
				);
				panic!("")
			}
		}

		Ok(Self {
			blobstore_root: blob_storage_dir,
			blobstore_tmp: blob_tmp_dir,
			conn: Mutex::new(conn),
		})
	}
}
