use std::{
	error::Error,
	fmt::Display,
	path::{Path, PathBuf},
	str::FromStr,
};

use copper_ds_core::api::Dataset;
use copper_pipeline::api::{PipelineData, PipelineJobContext};
use sqlx::{sqlite::SqliteConnectOptions, Connection, Row, SqliteConnection, SqlitePool};
use tracing::{error, info};

mod blob;
mod meta;
mod pipe;

#[derive(Debug)]
pub enum LocalDatasetCreateError {
	AlreadyExists,
	DbError(Box<dyn Error + Send + Sync>),
	IoError(std::io::Error),
}

impl Display for LocalDatasetCreateError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::AlreadyExists => write!(f, "dataset directory already exists"),
			Self::DbError(_) => write!(f, "sql error"),
			Self::IoError(_) => write!(f, "i/o error"),
		}
	}
}

impl Error for LocalDatasetCreateError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::DbError(e) => Some(e.as_ref()),
			Self::IoError(e) => Some(e),
			_ => return None,
		}
	}
}

#[derive(Debug)]
pub enum LocalDatasetOpenError {
	NotDir,
	DbError(Box<dyn Error + Send + Sync>),
	IoError(std::io::Error),
	BadTmpDir,
}

impl Display for LocalDatasetOpenError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::NotDir => write!(f, "target path is not a directory"),
			Self::DbError(_) => write!(f, "sql error"),
			Self::IoError(_) => write!(f, "i/o error"),
			Self::BadTmpDir => write!(f, "tmp dir is not a directory"),
		}
	}
}

impl Error for LocalDatasetOpenError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::DbError(e) => Some(e.as_ref()),
			Self::IoError(e) => Some(e),
			_ => return None,
		}
	}
}

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
	pub async fn create(ds_root: &Path) -> Result<(), LocalDatasetCreateError> {
		info!(
			message = "Creating dataset",
			ds_type = "LocalDataset",
			ds_root = ?ds_root
		);

		// Init root dir
		if ds_root.exists() {
			return Err(LocalDatasetCreateError::AlreadyExists);
		} else {
			std::fs::create_dir(ds_root).map_err(LocalDatasetCreateError::IoError)?;
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
			.map_err(|e| LocalDatasetCreateError::DbError(Box::new(e)))?;
		sqlx::query(include_str!("./init.sql"))
			.execute(&mut conn)
			.await
			.map_err(|e| LocalDatasetCreateError::DbError(Box::new(e)))?;

		sqlx::query("INSERT INTO meta_meta (var, val) VALUES (?, ?);")
			.bind("copper_version")
			.bind(env!("CARGO_PKG_VERSION"))
			.execute(&mut conn)
			.await
			.map_err(|e| LocalDatasetCreateError::DbError(Box::new(e)))?;

		// Initialize blob store
		let blob_storage_dir_absolute = ds_root.join(blob_storage_dir);
		let blob_tmp_dir_absolute = ds_root.join(blob_tmp_dir);
		std::fs::create_dir(blob_storage_dir_absolute).map_err(LocalDatasetCreateError::IoError)?;
		std::fs::create_dir(blob_tmp_dir_absolute).map_err(LocalDatasetCreateError::IoError)?;

		sqlx::query("INSERT INTO meta_meta (var, val) VALUES (?,?), (?,?);")
			.bind("blob_storage_dir")
			.bind(blob_storage_dir)
			.bind("blob_tmp_dir")
			.bind(blob_tmp_dir)
			.execute(&mut conn)
			.await
			.map_err(|e| LocalDatasetCreateError::DbError(Box::new(e)))?;

		Ok(())
	}

	pub async fn open(ds_root: &Path) -> Result<Self, LocalDatasetOpenError> {
		info!(
			message = "Opening dataset",
			ds_type = "LocalDataset",
			ds_root = ?ds_root,
		);

		if !ds_root.is_dir() {
			return Err(LocalDatasetOpenError::NotDir);
		}

		let db_file = ds_root.join("dataset.sqlite");
		let db_addr = format!("sqlite:{}?mode=rw", db_file.to_str().unwrap());

		let pool = SqlitePool::connect_with(
			SqliteConnectOptions::from_str(&db_addr)
				.map_err(|e| LocalDatasetOpenError::DbError(Box::new(e)))?
				// Disable statement cache. Each connection in this pool will have its own statement cache,
				// so the cache-clearing we do in the code below won't clear all statement caches.
				.statement_cache_capacity(0)
				.synchronous(sqlx::sqlite::SqliteSynchronous::Extra),
		)
		.await
		.map_err(|e| LocalDatasetOpenError::DbError(Box::new(e)))?;

		let mut conn = pool
			.acquire()
			.await
			.map_err(|e| LocalDatasetOpenError::DbError(Box::new(e)))?;

		// TODO: check version, blobstore dir

		let blob_storage_dir = ds_root.join({
			let res = sqlx::query("SELECT val FROM meta_meta WHERE var=\"blob_storage_dir\";")
				.fetch_one(&mut *conn)
				.await
				.map_err(|e| LocalDatasetOpenError::DbError(Box::new(e)))?;

			res.get::<String, _>("val")
		});

		let blob_tmp_dir = ds_root.join({
			let res = sqlx::query("SELECT val FROM meta_meta WHERE var=\"blob_tmp_dir\";")
				.fetch_one(&mut *conn)
				.await
				.map_err(|e| LocalDatasetOpenError::DbError(Box::new(e)))?;
			res.get::<String, _>("val")
		});

		let blob_tmp_dir_absolute = ds_root.join(&blob_tmp_dir);
		if blob_tmp_dir_absolute.exists() {
			if blob_tmp_dir_absolute.is_dir() {
				info!(
					message = "Deleting old blob tmp dir",
					directory = ?blob_tmp_dir_absolute
				);
				std::fs::remove_dir_all(&blob_tmp_dir_absolute)
					.map_err(LocalDatasetOpenError::IoError)?;
				std::fs::create_dir(blob_tmp_dir_absolute)
					.map_err(LocalDatasetOpenError::IoError)?;
			} else {
				error!(
					message = "Blobstore tmp dir is not a directory!",
					blob_tmp_dir = ?blob_tmp_dir_absolute
				);
				return Err(LocalDatasetOpenError::BadTmpDir);
			}
		}

		Ok(Self {
			blobstore_root: blob_storage_dir,
			blobstore_tmp: blob_tmp_dir,
			pool,
		})
	}
}
