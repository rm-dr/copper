use futures::executor::block_on;
use serde_with::{DeserializeFromStr, SerializeDisplay};
use smartstring::{LazyCompact, SmartString};
use sqlx::{Connection, Row};
use std::{fmt::Display, path::PathBuf, str::FromStr, sync::Arc};
use ufo_ds_core::api::Dataset;
use ufo_ds_impl::local::LocalDataset;
use ufo_pipeline_nodes::nodetype::UFONodeType;
use utoipa::ToSchema;

use super::{errors::CreateDatasetError, MainDB};

#[derive(Debug)]
pub struct DatasetEntry {
	pub name: SmartString<LazyCompact>,
	pub ds_type: DatasetType,
	pub path: PathBuf,
}

#[derive(Debug, SerializeDisplay, DeserializeFromStr, ToSchema)]
pub enum DatasetType {
	Local,
}

impl Display for DatasetType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Local => write!(f, "Local"),
		}
	}
}

impl FromStr for DatasetType {
	type Err = String;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"Local" => Self::Local,
			_ => return Err(format!("Unknown dataset type `{s}`")),
		})
	}
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
				LocalDataset::create(&self.config.dataset_dir.join(&path)).unwrap();
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
			.bind(entry.ds_type.to_string())
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

		return Ok(res
			.into_iter()
			.map(|x| DatasetEntry {
				name: x.get::<String, _>("ds_name").into(),
				ds_type: DatasetType::from_str(&x.get::<String, _>("ds_type")).unwrap(),
				path: x.get::<String, _>("ds_path").into(),
			})
			.collect());
	}

	pub fn get_dataset(
		&self,
		dataset_name: &str,
	) -> Result<Option<Arc<dyn Dataset<UFONodeType>>>, sqlx::Error> {
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
				ds_type: DatasetType::from_str(&res.get::<String, _>("ds_type")).unwrap(),
				path: res.get::<String, _>("ds_path").into(),
			},
		};

		Ok(Some(match entry.ds_type {
			DatasetType::Local => {
				Arc::new(LocalDataset::open(&self.config.dataset_dir.join(entry.path)).unwrap())
			}
		}))
	}

	pub fn del_dataset(&self, dataset_name: &str) -> Result<(), sqlx::Error> {
		let mut conn = self.conn.lock().unwrap();

		let res = block_on(
			sqlx::query("SELECT ds_name, ds_type, ds_path FROM datasets WHERE ds_name=?;")
				.bind(dataset_name)
				.fetch_one(&mut *conn),
		);

		let entry = match res {
			Err(sqlx::Error::RowNotFound) => panic!(),
			Err(e) => return Err(e),
			Ok(res) => DatasetEntry {
				name: res.get::<String, _>("ds_name").into(),
				ds_type: DatasetType::from_str(&res.get::<String, _>("ds_type")).unwrap(),
				path: res.get::<String, _>("ds_path").into(),
			},
		};

		match entry.ds_type {
			DatasetType::Local => {
				std::fs::remove_dir_all(&self.config.dataset_dir.join(&entry.path)).unwrap();
			}
		};

		let res = block_on(
			sqlx::query("DELETE FROM datasets WHERE ds_name=?;")
				.bind(dataset_name)
				.execute(&mut *conn),
		)
		.unwrap();

		Ok(())
	}
}
