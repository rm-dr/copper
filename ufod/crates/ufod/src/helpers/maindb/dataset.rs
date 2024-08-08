use futures::executor::block_on;
use smartstring::{LazyCompact, SmartString};
use sqlx::{Connection, Row};
use std::{fmt::Display, path::PathBuf, str::FromStr};
use ufo_ds_impl::local::LocalDataset;

use super::{errors::CreateDatasetError, MainDB};

pub struct DatasetEntry {
	name: SmartString<LazyCompact>,
	ds_type: DatasetType,
	path: PathBuf,
}

#[derive(Debug)]
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
	type Err = ();
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"Local" => Self::Local,
			_ => return Err(()),
		})
	}
}

impl MainDB {
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
}
