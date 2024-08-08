use copper_ds_impl::{local::LocalDataset, DatasetType};
use copper_util::names::clean_name;
use rand::{distributions::Alphanumeric, Rng};
use smartstring::{LazyCompact, SmartString};
use sqlx::{Connection, Row, SqlitePool};
use std::{collections::BTreeMap, path::PathBuf, sync::Arc};
use tokio::sync::Mutex;
use tracing::{debug, info};

use crate::config::CopperConfig;

pub mod errors;
use errors::{CreateDatasetError, RenameDatasetError};

#[derive(Debug)]
pub struct DatasetEntry {
	pub name: SmartString<LazyCompact>,
	pub ds_type: DatasetType,
	pub path: PathBuf,
}

pub struct DatasetProvider {
	pool: SqlitePool,
	config: Arc<CopperConfig>,
	open_datasets: Mutex<BTreeMap<SmartString<LazyCompact>, Arc<LocalDataset>>>,
}

impl DatasetProvider {
	pub(super) fn new(pool: SqlitePool, config: Arc<CopperConfig>) -> Self {
		Self {
			pool,
			config,
			open_datasets: Mutex::new(BTreeMap::new()),
		}
	}
}

impl DatasetProvider {
	pub async fn new_dataset(
		&self,
		name: &str,
		ds_type: DatasetType,
	) -> Result<(), CreateDatasetError> {
		let name = clean_name(name).map_err(CreateDatasetError::BadName)?;

		// Make sure this name is new
		let datasets = self
			.get_datasets()
			.await
			.map_err(|e| CreateDatasetError::DbError(Box::new(e)))?;
		if datasets.iter().any(|x| x.name == name) {
			return Err(CreateDatasetError::AlreadyExists);
		}

		debug!(
			message = "Creating dataset",
			name = name,
			dataset_type = ?ds_type
		);

		// generate new unique dir name
		let new_file_name = loop {
			let name: String = rand::thread_rng()
				.sample_iter(&Alphanumeric)
				.take(8)
				.map(char::from)
				.collect();

			let path = self.config.paths.dataset_dir.join(&name);
			if !path.exists() {
				break PathBuf::from(name);
			}
		};

		// Save this dataset in our index
		let entry = DatasetEntry {
			name: name.into(),
			ds_type,
			path: new_file_name,
		};

		// Make this dataset
		match ds_type {
			DatasetType::Local => {
				LocalDataset::create(&self.config.paths.dataset_dir.join(&entry.path))
					.await
					.unwrap();
			}
		}

		// Start transaction
		let mut conn = self
			.pool
			.acquire()
			.await
			.map_err(|e| CreateDatasetError::DbError(Box::new(e)))?;
		let mut t = conn
			.begin()
			.await
			.map_err(|e| CreateDatasetError::DbError(Box::new(e)))?;

		sqlx::query(
			"
			INSERT INTO datasets (
				ds_name, ds_type, ds_path
			) VALUES (?, ?, ?);
			",
		)
		.bind(entry.name.as_str())
		.bind(serde_json::to_string(&entry.ds_type).unwrap())
		.bind(entry.path.to_str().unwrap())
		.execute(&mut *t)
		.await
		.map_err(|e| CreateDatasetError::DbError(Box::new(e)))?;

		t.commit().await.unwrap();

		info!(
			message = "Created dataset",
			entry = ?entry
		);

		Ok(())
	}

	pub async fn get_datasets(&self) -> Result<Vec<DatasetEntry>, sqlx::Error> {
		let mut conn = self.pool.acquire().await?;
		let res = sqlx::query("SELECT ds_name, ds_type, ds_path FROM datasets ORDER BY id;")
			.fetch_all(&mut *conn)
			.await?;

		// TODO: one "open dataset" method
		return Ok(res
			.into_iter()
			.map(|x| DatasetEntry {
				name: x.get::<String, _>("ds_name").into(),
				ds_type: serde_json::from_str(&x.get::<String, _>("ds_type")).unwrap(),
				path: x.get::<String, _>("ds_path").into(),
			})
			.collect());
	}

	pub async fn get_dataset(
		&self,
		dataset_name: &str,
	) -> Result<Option<Arc<LocalDataset>>, sqlx::Error> {
		let mut ods = self.open_datasets.lock().await;

		// If this dataset is already open, we have nothing to do
		if let Some(ds) = ods.iter().find(|x| x.0 == dataset_name) {
			return Ok(Some(ds.1.clone()));
		}

		let mut conn = self.pool.acquire().await?;
		let res = sqlx::query("SELECT ds_name, ds_type, ds_path FROM datasets WHERE ds_name=?;")
			.bind(dataset_name)
			.fetch_one(&mut *conn)
			.await;

		let entry = match res {
			Err(sqlx::Error::RowNotFound) => return Ok(None),
			Err(e) => return Err(e),
			Ok(res) => DatasetEntry {
				name: res.get::<String, _>("ds_name").into(),
				ds_type: serde_json::from_str(&res.get::<String, _>("ds_type")).unwrap(),
				path: res.get::<String, _>("ds_path").into(),
			},
		};

		let ds = Arc::new(
			LocalDataset::open(&self.config.paths.dataset_dir.join(entry.path))
				.await
				.unwrap(),
		);

		ods.insert(entry.name.clone(), ds.clone());

		Ok(Some(match entry.ds_type {
			DatasetType::Local => ds,
		}))
	}

	pub async fn del_dataset(&self, dataset_name: &str) -> Result<(), sqlx::Error> {
		// If this dataset is already open, close it
		let mut ods_lock = self.open_datasets.lock().await;
		ods_lock.remove(dataset_name);
		drop(ods_lock);

		debug!(message = "Deleting dataset", name = dataset_name);

		// Start transaction
		let mut conn = self.pool.acquire().await?;
		let mut t = conn.begin().await?;

		let res = sqlx::query("SELECT ds_name, ds_type, ds_path FROM datasets WHERE ds_name=?;")
			.bind(dataset_name)
			.fetch_one(&mut *t)
			.await;

		let entry = match res {
			Err(sqlx::Error::RowNotFound) => panic!(),
			Err(e) => return Err(e),
			Ok(res) => DatasetEntry {
				name: res.get::<String, _>("ds_name").into(),
				ds_type: serde_json::from_str(&res.get::<String, _>("ds_type")).unwrap(),
				path: res.get::<String, _>("ds_path").into(),
			},
		};

		match entry.ds_type {
			DatasetType::Local => {
				std::fs::remove_dir_all(self.config.paths.dataset_dir.join(&entry.path)).unwrap();
			}
		};

		sqlx::query("DELETE FROM datasets WHERE ds_name=?;")
			.bind(dataset_name)
			.execute(&mut *t)
			.await?;

		t.commit().await?;

		info!(message = "Deleted dataset", name = dataset_name,);

		Ok(())
	}

	pub async fn rename_dataset(
		&self,
		old_name: &str,
		new_name: &str,
	) -> Result<(), RenameDatasetError> {
		let new_name = clean_name(new_name).map_err(RenameDatasetError::BadName)?;

		// Make sure this name is new
		let datasets = self
			.get_datasets()
			.await
			.map_err(|e| RenameDatasetError::DbError(Box::new(e)))?;
		if datasets.iter().any(|x| x.name == new_name) {
			return Err(RenameDatasetError::AlreadyExists);
		}

		debug!(message = "Renaming dataset", old_name, new_name);

		// If this dataset is already open, close it
		let mut ods_lock = self.open_datasets.lock().await;
		ods_lock.remove(old_name);
		drop(ods_lock);

		let mut conn = self
			.pool
			.acquire()
			.await
			.map_err(|e| RenameDatasetError::DbError(Box::new(e)))?;

		sqlx::query(
			"
			UPDATE datasets
			SET ds_name=?
			WHERE ds_name=?;
			",
		)
		.bind(new_name)
		.bind(old_name)
		.execute(&mut *conn)
		.await
		.map_err(|e| RenameDatasetError::DbError(Box::new(e)))?;

		Ok(())
	}
}
