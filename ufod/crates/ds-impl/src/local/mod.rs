use std::{
	path::{Path, PathBuf},
	str::FromStr,
};

use sqlx::{sqlite::SqliteConnectOptions, Connection, Row, SqliteConnection, SqlitePool};
use tracing::{error, info};
use ufo_ds_core::{api::Dataset, errors::MetastoreError};
use ufo_pipeline::api::{PipelineData, PipelineJobContext};

mod blob;
mod meta;
mod pipe;

pub struct LocalDataset {
	pool: SqlitePool,

	// Blobstore
	blobstore_root: PathBuf,
	blobstore_tmp: PathBuf,
}

impl<DataType: PipelineData, ContextType: PipelineJobContext<DataType>>
	Dataset<DataType, ContextType> for LocalDataset
{
}

impl LocalDataset {
	/// Create a new [`LocalDataset`].
	/// `db_root` must not exist and be empty.
	pub async fn create(ds_root: &Path) -> Result<(), MetastoreError> {
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
		let mut conn = SqliteConnection::connect(&db_addr)
			.await
			.map_err(|e| MetastoreError::DbError(Box::new(e)))?;
		sqlx::query(include_str!("./init.sql"))
			.execute(&mut conn)
			.await
			.unwrap();

		sqlx::query("INSERT INTO meta_meta (var, val) VALUES (?, ?);")
			.bind("ufo_version")
			.bind(env!("CARGO_PKG_VERSION"))
			.execute(&mut conn)
			.await
			.unwrap();

		// Initialize blob store
		let blob_storage_dir_absolute = ds_root.join(blob_storage_dir);
		let blob_tmp_dir_absolute = ds_root.join(blob_tmp_dir);
		std::fs::create_dir(blob_storage_dir_absolute).unwrap();
		std::fs::create_dir(blob_tmp_dir_absolute).unwrap();

		sqlx::query("INSERT INTO meta_meta (var, val) VALUES (?,?), (?,?);")
			.bind("blob_storage_dir")
			.bind(blob_storage_dir)
			.bind("blob_tmp_dir")
			.bind(blob_tmp_dir)
			.execute(&mut conn)
			.await
			.unwrap();

		Ok(())
	}

	pub async fn open(ds_root: &Path) -> Result<Self, ()> {
		info!(
			message = "Opening dataset",
			ds_root = ?ds_root
		);

		let db_file = ds_root.join("dataset.sqlite");
		let db_addr = format!("sqlite:{}?mode=rw", db_file.to_str().unwrap());

		let pool = SqlitePool::connect_with(
			SqliteConnectOptions::from_str(&db_addr)
				.unwrap()
				// Disable statement cache. Each connection in this pool will have its own statement cache,
				// so the cache-clearing we do in the code below won't clear all statement caches.
				.statement_cache_capacity(0)
				.synchronous(sqlx::sqlite::SqliteSynchronous::Extra),
		)
		.await
		.unwrap();
		let mut conn = pool.acquire().await.unwrap();

		// TODO: check version, blobstore dir

		let blob_storage_dir = ds_root.join({
			let res = sqlx::query("SELECT val FROM meta_meta WHERE var=\"blob_storage_dir\";")
				.fetch_one(&mut *conn)
				.await
				.unwrap();
			res.get::<String, _>("val")
		});

		let blob_tmp_dir = ds_root.join({
			let res = sqlx::query("SELECT val FROM meta_meta WHERE var=\"blob_tmp_dir\";")
				.fetch_one(&mut *conn)
				.await
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
				panic!()
			}
		}

		Ok(Self {
			blobstore_root: blob_storage_dir,
			blobstore_tmp: blob_tmp_dir,
			pool,
		})
	}
}
