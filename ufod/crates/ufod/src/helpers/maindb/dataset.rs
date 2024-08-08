use rand::{distributions::Alphanumeric, Rng};
use smartstring::{LazyCompact, SmartString};
use sqlx::{Connection, Row};
use std::{path::PathBuf, sync::Arc};
use ufo_ds_impl::{local::LocalDataset, DatasetType};

use super::{errors::CreateDatasetError, MainDB};

#[derive(Debug)]
pub struct DatasetEntry {
	pub name: SmartString<LazyCompact>,
	pub ds_type: DatasetType,
	pub path: PathBuf,
}

impl MainDB {
	pub async fn new_dataset(
		&self,
		name: &str,
		ds_type: DatasetType,
	) -> Result<(), CreateDatasetError> {
		// No empty names
		if name == "" {
			return Err(CreateDatasetError::BadName(
				"Dataset name cannot be empty".into(),
			));
		} else if name.trim() == "" {
			return Err(CreateDatasetError::BadName(
				"Dataset name cannot be whitespace".into(),
			));
		}

		// Make sure this name is new
		let datasets = self
			.get_datasets()
			.await
			.map_err(|e| CreateDatasetError::DbError(Box::new(e)))?;
		if datasets.iter().any(|x| x.name == name) {
			return Err(CreateDatasetError::AlreadyExists);
		}

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
		let mut conn_lock = self.conn.lock().await;
		let mut t = conn_lock
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

		Ok(())
	}

	pub async fn get_datasets(&self) -> Result<Vec<DatasetEntry>, sqlx::Error> {
		let mut conn = self.conn.lock().await;

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
		// If this dataset is already open, we have nothing to do
		if let Some(ds) = self
			.open_datasets
			.lock()
			.await
			.iter()
			.find(|x| x.0 == dataset_name)
		{
			return Ok(Some(ds.1.clone()));
		}

		let mut conn = self.conn.lock().await;

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

		self.open_datasets
			.lock()
			.await
			.push((entry.name.clone(), ds.clone()));

		Ok(Some(match entry.ds_type {
			DatasetType::Local => ds,
		}))
	}

	pub async fn del_dataset(&self, dataset_name: &str) -> Result<(), sqlx::Error> {
		// If this dataset is already open, close it
		let mut ods_lock = self.open_datasets.lock().await;
		if let Some((idx, _)) = ods_lock
			.iter()
			.enumerate()
			.find(|(_, x)| x.0 == dataset_name)
		{
			ods_lock.swap_remove(idx);
		}
		drop(ods_lock);

		// Start transaction
		let mut conn_lock = self.conn.lock().await;
		let mut t = conn_lock.begin().await?;

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
				std::fs::remove_dir_all(&self.config.paths.dataset_dir.join(&entry.path)).unwrap();
			}
		};

		sqlx::query("DELETE FROM datasets WHERE ds_name=?;")
			.bind(dataset_name)
			.execute(&mut *t)
			.await?;

		t.commit().await?;

		Ok(())
	}
}
