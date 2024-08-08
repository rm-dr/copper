use futures::executor::block_on;
use smartstring::{LazyCompact, SmartString};
use sqlx::{Connection, Row};
use std::{path::PathBuf, sync::Arc};
use ufo_ds_core::api::Dataset;
use ufo_ds_impl::{local::LocalDataset, DatasetType};
use ufo_pipeline_nodes::nodetype::UFONodeType;

use super::{errors::CreateDatasetError, MainDB};

#[derive(Debug)]
pub struct DatasetEntry {
	pub name: SmartString<LazyCompact>,
	pub ds_type: DatasetType,
	pub path: PathBuf,
}

impl MainDB {
	// TODO: escape instead, what about other languages?

	/// Check a dataset name, returning an error description
	/// or `None` if nothing is wrong.
	fn check_dataset_name(name: &str) -> Option<String> {
		let allowed_chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz1234567890-_";

		for c in name.chars() {
			if !allowed_chars.contains(c) {
				return Some(format!(
					"Invalid character `{c}`. A dataset name may only contain A-z, 0-9, _, and -"
				));
			}
		}

		return None;
	}

	pub fn new_dataset(&self, name: &str, ds_type: DatasetType) -> Result<(), CreateDatasetError> {
		// Make sure this name is valid
		if let Some(msg) = Self::check_dataset_name(name) {
			return Err(CreateDatasetError::BadName(msg));
		}

		// Make sure this name is new
		let datasets = self
			.get_datasets()
			.map_err(|e| CreateDatasetError::DbError(Box::new(e)))?;
		if datasets.iter().any(|x| x.name == name) {
			return Err(CreateDatasetError::AlreadyExists(name.into()));
		}

		// Make this dataset
		let path = match ds_type {
			DatasetType::Local => {
				let path = PathBuf::from(name);
				LocalDataset::create(&self.config.paths.dataset_dir.join(&path)).unwrap();
				path
			}
		};

		// Save this dataset in our index
		let entry = DatasetEntry {
			name: name.into(),
			ds_type,
			path,
		};

		// Start transaction
		let mut conn_lock = self.conn.lock().unwrap();
		let mut t =
			block_on(conn_lock.begin()).map_err(|e| CreateDatasetError::DbError(Box::new(e)))?;

		block_on(
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
			.execute(&mut *t),
		)
		.map_err(|e| CreateDatasetError::DbError(Box::new(e)))?;

		block_on(t.commit()).unwrap();

		Ok(())
	}

	pub fn get_datasets(&self) -> Result<Vec<DatasetEntry>, sqlx::Error> {
		let mut conn = self.conn.lock().unwrap();

		let res = block_on(
			sqlx::query("SELECT ds_name, ds_type, ds_path FROM datasets ORDER BY id;")
				.fetch_all(&mut *conn),
		)?;

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

	pub fn get_dataset(
		&self,
		dataset_name: &str,
	) -> Result<Option<Arc<dyn Dataset<UFONodeType>>>, sqlx::Error> {
		// If this dataset is already open, we have nothing to do
		if let Some(ds) = self
			.open_datasets
			.lock()
			.unwrap()
			.iter()
			.find(|x| x.0 == dataset_name)
		{
			return Ok(Some(ds.1.clone()));
		}

		let mut conn = self.conn.lock().unwrap();

		let res = block_on(
			sqlx::query("SELECT ds_name, ds_type, ds_path FROM datasets WHERE ds_name=?;")
				.bind(dataset_name)
				.fetch_one(&mut *conn),
		);

		let entry = match res {
			Err(sqlx::Error::RowNotFound) => return Ok(None),
			Err(e) => return Err(e),
			Ok(res) => DatasetEntry {
				name: res.get::<String, _>("ds_name").into(),
				ds_type: serde_json::from_str(&res.get::<String, _>("ds_type")).unwrap(),
				path: res.get::<String, _>("ds_path").into(),
			},
		};

		let ds =
			Arc::new(LocalDataset::open(&self.config.paths.dataset_dir.join(entry.path)).unwrap());

		self.open_datasets
			.lock()
			.unwrap()
			.push((entry.name.clone(), ds.clone()));

		Ok(Some(match entry.ds_type {
			DatasetType::Local => ds,
		}))
	}

	pub fn del_dataset(&self, dataset_name: &str) -> Result<(), sqlx::Error> {
		// If this dataset is already open, close it
		let mut ods_lock = self.open_datasets.lock().unwrap();
		if let Some((idx, _)) = ods_lock
			.iter()
			.enumerate()
			.find(|(_, x)| x.0 == dataset_name)
		{
			ods_lock.swap_remove(idx);
		}
		drop(ods_lock);

		// Start transaction
		let mut conn_lock = self.conn.lock().unwrap();
		let mut t = block_on(conn_lock.begin())?;

		let res = block_on(
			sqlx::query("SELECT ds_name, ds_type, ds_path FROM datasets WHERE ds_name=?;")
				.bind(dataset_name)
				.fetch_one(&mut *t),
		);

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

		block_on(
			sqlx::query("DELETE FROM datasets WHERE ds_name=?;")
				.bind(dataset_name)
				.execute(&mut *t),
		)?;

		block_on(t.commit())?;

		Ok(())
	}
}
