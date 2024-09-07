use crate::api::{
	data::DatasetDataStub,
	errors::{
		attribute::{
			AddAttributeError, DeleteAttributeError, GetAttributeError, RenameAttributeError,
		},
		dataset::{AddDatasetError, DeleteDatasetError, GetDatasetError, RenameDatasetError},
		itemclass::{
			AddItemclassError, DeleteItemclassError, GetItemclassError, RenameItemclassError,
		},
	},
	handles::{AttributeHandle, DatasetHandle, ItemclassHandle},
	AttributeInfo, AttributeOptions, DatabaseClient, DatasetInfo, ItemclassInfo,
};
use async_trait::async_trait;
use copper_util::mime::MimeType;
use serde::{Deserialize, Serialize};
use sqlx::{
	query::Query,
	sqlite::{SqliteArguments, SqliteRow},
	Connection, Row, Sqlite,
};
use std::sync::Arc;

use super::SqliteDatabase;

#[derive(Debug, Clone, Deserialize, Serialize)]
struct BlobJsonEncoded {
	url: String,
	mime: MimeType,
}

// SQL helper functions
impl SqliteDatabase {
	/*
	fn bind_storage<'a>(
		q: Query<'a, Sqlite, SqliteArguments<'a>>,
		storage: &'a mut DatasetData,
	) -> Result<Query<'a, Sqlite, SqliteArguments<'a>>, DatabaseError> {
		Ok(match storage {
			// We MUST bind something, even for null values.
			// If we don't, the null value's '?' won't be used
			// and all following fields will be shifted left.
			DatasetData::None(_) => q.bind(None::<u32>),
			DatasetData::Text(s) => q.bind(s.as_str()),
			DatasetData::Reference { item, .. } => q.bind(u32::from(*item)),
			DatasetData::Blob { url, mime } => q.bind(
				serde_json::to_string(&BlobJsonEncoded {
					url: url.clone(),
					mime: mime.clone(),
				})
				.unwrap(),
			),
			DatasetData::Boolean(x) => q.bind(*x),
			DatasetData::Hash { data, .. } => q.bind(&**data),

			DatasetData::Float {
				value,
				is_non_negative,
			} => {
				if *is_non_negative && *value < 0.0 {
					return Err(DatabaseError::NonNegativeViolated);
				}
				q.bind(*value)
			}

			DatasetData::Integer {
				value,
				is_non_negative,
			} => {
				if *is_non_negative && *value < 0 {
					return Err(DatabaseError::NonNegativeViolated);
				}
				q.bind(*value)
			}
		})
	}
	*/

	/*
	fn read_storage(row: &SqliteRow, attr: &AttributeInfo) -> DatasetData {
		let col_name = Self::get_column_name(attr.handle);
		return match attr.data_type {
			DatasetDataStub::Float { is_non_negative } => row
				.get::<Option<_>, _>(&col_name[..])
				.map(|value| DatasetData::Float {
					is_non_negative,
					value,
				})
				.unwrap_or(DatasetData::None(attr.data_type)),

			DatasetDataStub::Boolean => row
				.get::<Option<_>, _>(&col_name[..])
				.map(DatasetData::Boolean)
				.unwrap_or(DatasetData::None(attr.data_type)),

			DatasetDataStub::Integer { is_non_negative } => row
				.get::<Option<_>, _>(&col_name[..])
				.map(|value| DatasetData::Integer {
					is_non_negative,
					value,
				})
				.unwrap_or(DatasetData::None(attr.data_type)),

			DatasetDataStub::Text => row
				.get::<Option<String>, _>(&col_name[..])
				.map(SmartString::from)
				.map(Arc::new)
				.map(DatasetData::Text)
				.unwrap_or(DatasetData::None(attr.data_type)),

			DatasetDataStub::Reference { class } => row
				.get::<Option<_>, _>(&col_name[..])
				.map(|item: u32| DatasetData::Reference {
					class,
					item: item.into(),
				})
				.unwrap_or(DatasetData::None(attr.data_type)),

			DatasetDataStub::Hash { hash_type } => row
				.get::<Option<_>, _>(&col_name[..])
				.map(|item| DatasetData::Hash {
					format: hash_type,
					data: Arc::new(item),
				})
				.unwrap_or(DatasetData::None(attr.data_type)),

			DatasetDataStub::Blob => row
				.get::<Option<_>, _>(&col_name[..])
				.map(|item: String| DatasetData::Blob {
					url: item.into(),
					mime: MimeType::Avif,
				})
				.unwrap_or(DatasetData::None(attr.data_type)),
		};
	}
	*/
}

#[async_trait]
impl DatabaseClient for SqliteDatabase {
	//
	// MARK: Dataset
	//

	async fn add_dataset(&self, name: &str) -> Result<DatasetHandle, AddDatasetError> {
		// Start transaction
		let mut conn = self
			.pool
			.acquire()
			.await
			.map_err(|e| AddDatasetError::DbError(Box::new(e)))?;
		let mut t = conn
			.begin()
			.await
			.map_err(|e| AddDatasetError::DbError(Box::new(e)))?;

		let res = sqlx::query("INSERT INTO dataset (pretty_name) VALUES (?);")
			.bind(name)
			.execute(&mut *t)
			.await;

		t.commit()
			.await
			.map_err(|e| AddDatasetError::DbError(Box::new(e)))?;

		let new_handle: DatasetHandle = match res {
			Ok(x) => u32::try_from(x.last_insert_rowid()).unwrap().into(),
			Err(sqlx::Error::Database(e)) => {
				if e.is_unique_violation() {
					return Err(AddDatasetError::AlreadyExists);
				} else {
					let e = Box::new(sqlx::Error::Database(e));
					return Err(AddDatasetError::DbError(e));
				}
			}
			Err(e) => return Err(AddDatasetError::DbError(Box::new(e))),
		};

		return Ok(new_handle);
	}

	async fn get_dataset(&self, dataset: DatasetHandle) -> Result<DatasetInfo, GetDatasetError> {
		let mut conn = self
			.pool
			.acquire()
			.await
			.map_err(|e| GetDatasetError::DbError(Box::new(e)))?;

		let res = sqlx::query("SELECT * FROM dataset WHERE id=?;")
			.bind(u32::from(dataset))
			.fetch_one(&mut *conn)
			.await;

		return match res {
			Err(sqlx::Error::RowNotFound) => Err(GetDatasetError::NotFound),
			Err(e) => Err(GetDatasetError::DbError(Box::new(e))),
			Ok(res) => Ok(DatasetInfo {
				handle: res.get::<u32, _>("id").into(),
				name: res.get::<String, _>("pretty_name").into(),
			}),
		};
	}

	async fn rename_dataset(
		&self,
		dataset: DatasetHandle,
		new_name: &str,
	) -> Result<(), RenameDatasetError> {
		let mut conn = self
			.pool
			.acquire()
			.await
			.map_err(|e| RenameDatasetError::DbError(Box::new(e)))?;
		let mut t = conn
			.begin()
			.await
			.map_err(|e| RenameDatasetError::DbError(Box::new(e)))?;

		let res = sqlx::query("UPDATE dataset SET pretty_name=? WHERE id=?;")
			.bind(new_name)
			.bind(u32::from(dataset))
			.execute(&mut *t)
			.await;

		t.commit()
			.await
			.map_err(|e| RenameDatasetError::DbError(Box::new(e)))?;

		return match res {
			Err(e) => Err(RenameDatasetError::DbError(Box::new(e))),
			Ok(_) => Ok(()),
		};
	}

	async fn del_dataset(&self, dataset: DatasetHandle) -> Result<(), DeleteDatasetError> {
		let mut conn = self
			.pool
			.acquire()
			.await
			.map_err(|e| DeleteDatasetError::DbError(Box::new(e)))?;
		let mut t = conn
			.begin()
			.await
			.map_err(|e| DeleteDatasetError::DbError(Box::new(e)))?;

		// This also deletes all attributes, etc,
		// since they're marked with ON DELETE CASCADE.
		sqlx::query("DELETE FROM dataset WHERE id=?;")
			.bind(u32::from(dataset))
			.execute(&mut *t)
			.await
			.map_err(|e| DeleteDatasetError::DbError(Box::new(e)))?;

		t.commit()
			.await
			.map_err(|e| DeleteDatasetError::DbError(Box::new(e)))?;

		return Ok(());
	}

	//
	// MARK: Itemclass
	//

	async fn add_itemclass(
		&self,
		in_dataset: DatasetHandle,
		name: &str,
	) -> Result<ItemclassHandle, AddItemclassError> {
		// Start transaction
		let mut conn = self
			.pool
			.acquire()
			.await
			.map_err(|e| AddItemclassError::DbError(Box::new(e)))?;
		let mut t = conn
			.begin()
			.await
			.map_err(|e| AddItemclassError::DbError(Box::new(e)))?;

		let res = sqlx::query("INSERT INTO itemclass (dataset_id, pretty_name) VALUES (?, ?);")
			.bind(u32::from(in_dataset))
			.bind(name)
			.execute(&mut *t)
			.await;

		t.commit()
			.await
			.map_err(|e| AddItemclassError::DbError(Box::new(e)))?;

		let new_handle: ItemclassHandle = match res {
			Ok(x) => u32::try_from(x.last_insert_rowid()).unwrap().into(),
			Err(sqlx::Error::Database(e)) => {
				if e.is_foreign_key_violation() {
					return Err(AddItemclassError::NoSuchDataset);
				} else {
					let e = Box::new(sqlx::Error::Database(e));
					return Err(AddItemclassError::DbError(e));
				}
			}
			Err(e) => return Err(AddItemclassError::DbError(Box::new(e))),
		};
		return Ok(new_handle);
	}

	async fn get_itemclass(
		&self,
		itemclass: ItemclassHandle,
	) -> Result<ItemclassInfo, GetItemclassError> {
		let mut conn = self
			.pool
			.acquire()
			.await
			.map_err(|e| GetItemclassError::DbError(Box::new(e)))?;

		let res = sqlx::query("SELECT * FROM itemclass WHERE id=?;")
			.bind(u32::from(itemclass))
			.fetch_one(&mut *conn)
			.await;

		return match res {
			Err(sqlx::Error::RowNotFound) => Err(GetItemclassError::NotFound),
			Err(e) => Err(GetItemclassError::DbError(Box::new(e))),
			Ok(res) => Ok(ItemclassInfo {
				handle: res.get::<u32, _>("id").into(),
				name: res.get::<String, _>("pretty_name").into(),
			}),
		};
	}

	async fn rename_itemclass(
		&self,
		itemclass: ItemclassHandle,
		new_name: &str,
	) -> Result<(), RenameItemclassError> {
		let mut conn = self
			.pool
			.acquire()
			.await
			.map_err(|e| RenameItemclassError::DbError(Box::new(e)))?;
		let mut t = conn
			.begin()
			.await
			.map_err(|e| RenameItemclassError::DbError(Box::new(e)))?;

		let res = sqlx::query("UPDATE itemclass SET pretty_name=? WHERE id=?;")
			.bind(new_name)
			.bind(u32::from(itemclass))
			.execute(&mut *t)
			.await;

		t.commit()
			.await
			.map_err(|e| RenameItemclassError::DbError(Box::new(e)))?;

		return match res {
			Err(e) => Err(RenameItemclassError::DbError(Box::new(e))),
			Ok(_) => Ok(()),
		};
	}

	async fn del_itemclass(&self, itemclass: ItemclassHandle) -> Result<(), DeleteItemclassError> {
		let mut conn = self
			.pool
			.acquire()
			.await
			.map_err(|e| DeleteItemclassError::DbError(Box::new(e)))?;
		let mut t = conn
			.begin()
			.await
			.map_err(|e| DeleteItemclassError::DbError(Box::new(e)))?;

		// This also deletes all itemclasses, attributes, etc,
		// since they're marked with ON DELETE CASCADE.
		sqlx::query("DELETE FROM itemclass WHERE id=?;")
			.bind(u32::from(itemclass))
			.execute(&mut *t)
			.await
			.map_err(|e| DeleteItemclassError::DbError(Box::new(e)))?;

		t.commit()
			.await
			.map_err(|e| DeleteItemclassError::DbError(Box::new(e)))?;

		return Ok(());
	}

	//
	// MARK: Attribute
	//

	async fn add_attribute(
		&self,
		in_itemclass: ItemclassHandle,
		name: &str,
		with_type: DatasetDataStub,
		options: AttributeOptions,
	) -> Result<AttributeHandle, AddAttributeError> {
		// Start transaction
		let mut conn = self
			.pool
			.acquire()
			.await
			.map_err(|e| AddAttributeError::DbError(Box::new(e)))?;
		let mut t = conn
			.begin()
			.await
			.map_err(|e| AddAttributeError::DbError(Box::new(e)))?;

		let res = sqlx::query(
			"INSERT INTO attribute(itemclass_id, attr_order, pretty_name, data_type, is_unique, is_not_null)
			SELECT ?, COALESCE(MAX(attr_order) + 1 , 0), ?, ?, ?, ? FROM attribute WHERE itemclass_id=?;",
		)
		.bind(u32::from(in_itemclass))
		.bind(name)
		.bind(serde_json::to_string(&with_type).unwrap())
		.bind(options.unique)
		.bind(options.is_not_null)
		.bind(u32::from(in_itemclass))
		.execute(&mut *t)
		.await;

		t.commit()
			.await
			.map_err(|e| AddAttributeError::DbError(Box::new(e)))?;

		let new_handle: AttributeHandle = match res {
			Ok(x) => u32::try_from(x.last_insert_rowid()).unwrap().into(),
			Err(sqlx::Error::Database(e)) => {
				if e.is_foreign_key_violation() {
					return Err(AddAttributeError::NoSuchItemclass);
				} else {
					let e = Box::new(sqlx::Error::Database(e));
					return Err(AddAttributeError::DbError(e));
				}
			}
			Err(e) => return Err(AddAttributeError::DbError(Box::new(e))),
		};

		return Ok(new_handle);
	}

	async fn get_attribute(
		&self,
		attribute: AttributeHandle,
	) -> Result<AttributeInfo, GetAttributeError> {
		let mut conn = self
			.pool
			.acquire()
			.await
			.map_err(|e| GetAttributeError::DbError(Box::new(e)))?;

		let res = sqlx::query("SELECT * FROM itemclass WHERE id=?;")
			.bind(u32::from(attribute))
			.fetch_one(&mut *conn)
			.await;

		return match res {
			Err(sqlx::Error::RowNotFound) => Err(GetAttributeError::NotFound),
			Err(e) => Err(GetAttributeError::DbError(Box::new(e))),
			Ok(res) => Ok(AttributeInfo {
				handle: res.get::<u32, _>("id").into(),
				itemclass: res.get::<u32, _>("id").into(),
				order: res.get::<u32, _>("attr_order").into(),
				name: res.get::<String, _>("pretty_name").into(),
				data_type: serde_json::from_str(res.get::<&str, _>("data_type")).unwrap(),
				is_unique: res.get("is_unique"),
				is_not_null: res.get("is_not_null"),
			}),
		};
	}

	async fn rename_attribute(
		&self,
		attribute: AttributeHandle,
		new_name: &str,
	) -> Result<(), RenameAttributeError> {
		let mut conn = self
			.pool
			.acquire()
			.await
			.map_err(|e| RenameAttributeError::DbError(Box::new(e)))?;
		let mut t = conn
			.begin()
			.await
			.map_err(|e| RenameAttributeError::DbError(Box::new(e)))?;

		let res = sqlx::query("UPDATE attribute SET pretty_name=? WHERE id=?;")
			.bind(new_name)
			.bind(u32::from(attribute))
			.execute(&mut *t)
			.await;

		t.commit()
			.await
			.map_err(|e| RenameAttributeError::DbError(Box::new(e)))?;

		return match res {
			Err(e) => Err(RenameAttributeError::DbError(Box::new(e))),
			Ok(_) => Ok(()),
		};
	}

	async fn del_attribute(&self, attribute: AttributeHandle) -> Result<(), DeleteAttributeError> {
		let mut conn = self
			.pool
			.acquire()
			.await
			.map_err(|e| DeleteAttributeError::DbError(Box::new(e)))?;
		let mut t = conn
			.begin()
			.await
			.map_err(|e| DeleteAttributeError::DbError(Box::new(e)))?;

		// This also deletes all attribute entries, etc,
		// since they're marked with ON DELETE CASCADE.
		sqlx::query("DELETE FROM attribute WHERE id=?;")
			.bind(u32::from(attribute))
			.execute(&mut *t)
			.await
			.map_err(|e| DeleteAttributeError::DbError(Box::new(e)))?;

		t.commit()
			.await
			.map_err(|e| DeleteAttributeError::DbError(Box::new(e)))?;

		return Ok(());
	}
}
