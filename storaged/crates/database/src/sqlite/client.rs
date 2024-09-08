use std::collections::BTreeMap;

use crate::api::{
	client::{AttributeOptions, DatabaseClient},
	data::{AttrData, AttrDataStub},
	errors::{
		attribute::{
			AddAttributeError, DeleteAttributeError, GetAttributeError, RenameAttributeError,
		},
		class::{AddClassError, DeleteClassError, GetClassError, RenameClassError},
		dataset::{AddDatasetError, DeleteDatasetError, GetDatasetError, RenameDatasetError},
		item::{AddItemError, DeleteItemError, GetItemError},
	},
	handles::{AttributeId, ClassId, DatasetId, ItemId},
	info::{AttributeInfo, ClassInfo, DatasetInfo, ItemInfo},
};
use async_trait::async_trait;
use copper_util::{mime::MimeType, names::check_name};
use serde::{Deserialize, Serialize};
use sqlx::{Connection, Row};

use super::SqliteDatabaseClient;

#[derive(Debug, Clone, Deserialize, Serialize)]
struct BlobJsonEncoded {
	url: String,
	mime: MimeType,
}

// SQL helper functions
impl SqliteDatabaseClient {
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
impl DatabaseClient for SqliteDatabaseClient {
	//
	// MARK: Dataset
	//

	async fn add_dataset(&self, name: &str) -> Result<DatasetId, AddDatasetError> {
		match check_name(name) {
			Ok(()) => {}
			Err(e) => return Err(AddDatasetError::NameError(e)),
		}

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

		let new_handle: DatasetId = match res {
			Ok(x) => u32::try_from(x.last_insert_rowid()).unwrap().into(),
			Err(sqlx::Error::Database(e)) => {
				if e.is_unique_violation() {
					return Err(AddDatasetError::UniqueViolation);
				} else {
					let e = Box::new(sqlx::Error::Database(e));
					return Err(AddDatasetError::DbError(e));
				}
			}
			Err(e) => return Err(AddDatasetError::DbError(Box::new(e))),
		};

		return Ok(new_handle);
	}

	async fn get_dataset(&self, dataset: DatasetId) -> Result<DatasetInfo, GetDatasetError> {
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
				id: res.get::<u32, _>("id").into(),
				name: res.get::<String, _>("pretty_name").into(),
			}),
		};
	}

	async fn rename_dataset(
		&self,
		dataset: DatasetId,
		new_name: &str,
	) -> Result<(), RenameDatasetError> {
		match check_name(new_name) {
			Ok(()) => {}
			Err(e) => return Err(RenameDatasetError::NameError(e)),
		}

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
			Ok(_) => Ok(()),
			Err(sqlx::Error::Database(e)) => {
				if e.is_unique_violation() {
					Err(RenameDatasetError::UniqueViolation)
				} else {
					let e = Box::new(sqlx::Error::Database(e));
					Err(RenameDatasetError::DbError(e))
				}
			}
			Err(e) => Err(RenameDatasetError::DbError(Box::new(e))),
		};
	}

	async fn del_dataset(&self, dataset: DatasetId) -> Result<(), DeleteDatasetError> {
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
	// MARK: Class
	//

	async fn add_class(&self, in_dataset: DatasetId, name: &str) -> Result<ClassId, AddClassError> {
		match check_name(name) {
			Ok(()) => {}
			Err(e) => return Err(AddClassError::NameError(e)),
		}

		// Start transaction
		let mut conn = self
			.pool
			.acquire()
			.await
			.map_err(|e| AddClassError::DbError(Box::new(e)))?;
		let mut t = conn
			.begin()
			.await
			.map_err(|e| AddClassError::DbError(Box::new(e)))?;

		let res = sqlx::query("INSERT INTO class (dataset_id, pretty_name) VALUES (?, ?);")
			.bind(u32::from(in_dataset))
			.bind(name)
			.execute(&mut *t)
			.await;

		t.commit()
			.await
			.map_err(|e| AddClassError::DbError(Box::new(e)))?;

		let new_handle: ClassId = match res {
			Ok(x) => u32::try_from(x.last_insert_rowid()).unwrap().into(),
			Err(sqlx::Error::Database(e)) => {
				if e.is_foreign_key_violation() {
					return Err(AddClassError::NoSuchDataset);
				} else if e.is_unique_violation() {
					return Err(AddClassError::UniqueViolation);
				} else {
					let e = Box::new(sqlx::Error::Database(e));
					return Err(AddClassError::DbError(e));
				}
			}
			Err(e) => return Err(AddClassError::DbError(Box::new(e))),
		};
		return Ok(new_handle);
	}

	async fn get_class(&self, class: ClassId) -> Result<ClassInfo, GetClassError> {
		let mut conn = self
			.pool
			.acquire()
			.await
			.map_err(|e| GetClassError::DbError(Box::new(e)))?;

		let res = sqlx::query("SELECT * FROM class WHERE id=?;")
			.bind(u32::from(class))
			.fetch_one(&mut *conn)
			.await;

		return match res {
			Err(sqlx::Error::RowNotFound) => Err(GetClassError::NotFound),
			Err(e) => Err(GetClassError::DbError(Box::new(e))),
			Ok(res) => Ok(ClassInfo {
				id: res.get::<u32, _>("id").into(),
				name: res.get::<String, _>("pretty_name").into(),
			}),
		};
	}

	async fn rename_class(&self, class: ClassId, new_name: &str) -> Result<(), RenameClassError> {
		match check_name(new_name) {
			Ok(()) => {}
			Err(e) => return Err(RenameClassError::NameError(e)),
		}

		let mut conn = self
			.pool
			.acquire()
			.await
			.map_err(|e| RenameClassError::DbError(Box::new(e)))?;
		let mut t = conn
			.begin()
			.await
			.map_err(|e| RenameClassError::DbError(Box::new(e)))?;

		let res = sqlx::query("UPDATE class SET pretty_name=? WHERE id=?;")
			.bind(new_name)
			.bind(u32::from(class))
			.execute(&mut *t)
			.await;

		t.commit()
			.await
			.map_err(|e| RenameClassError::DbError(Box::new(e)))?;

		return match res {
			Err(sqlx::Error::Database(e)) => {
				if e.is_unique_violation() {
					Err(RenameClassError::UniqueViolation)
				} else {
					let e = Box::new(sqlx::Error::Database(e));
					Err(RenameClassError::DbError(e))
				}
			}

			Err(e) => Err(RenameClassError::DbError(Box::new(e))),

			Ok(_) => Ok(()),
		};
	}

	async fn del_class(&self, class: ClassId) -> Result<(), DeleteClassError> {
		let mut conn = self
			.pool
			.acquire()
			.await
			.map_err(|e| DeleteClassError::DbError(Box::new(e)))?;
		let mut t = conn
			.begin()
			.await
			.map_err(|e| DeleteClassError::DbError(Box::new(e)))?;

		// This also deletes all classes, attributes, etc,
		// since they're marked with ON DELETE CASCADE.
		sqlx::query("DELETE FROM class WHERE id=?;")
			.bind(u32::from(class))
			.execute(&mut *t)
			.await
			.map_err(|e| DeleteClassError::DbError(Box::new(e)))?;

		t.commit()
			.await
			.map_err(|e| DeleteClassError::DbError(Box::new(e)))?;

		return Ok(());
	}

	//
	// MARK: Attribute
	//

	async fn add_attribute(
		&self,
		in_class: ClassId,
		name: &str,
		with_type: AttrDataStub,
		options: AttributeOptions,
	) -> Result<AttributeId, AddAttributeError> {
		match check_name(name) {
			Ok(()) => {}
			Err(e) => return Err(AddAttributeError::NameError(e)),
		}

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
			"INSERT INTO attribute(class_id, attr_order, pretty_name, data_type, is_unique, is_not_null)
			SELECT ?, COALESCE(MAX(attr_order) + 1 , 0), ?, ?, ?, ? FROM attribute WHERE class_id=?;",
		)
		.bind(u32::from(in_class))
		.bind(name)
		.bind(serde_json::to_string(&with_type).unwrap())
		.bind(options.unique)
		.bind(options.is_not_null)
		.bind(u32::from(in_class))
		.execute(&mut *t)
		.await;

		t.commit()
			.await
			.map_err(|e| AddAttributeError::DbError(Box::new(e)))?;

		let new_handle: AttributeId = match res {
			Ok(x) => u32::try_from(x.last_insert_rowid()).unwrap().into(),
			Err(sqlx::Error::Database(e)) => {
				if e.is_foreign_key_violation() {
					return Err(AddAttributeError::NoSuchClass);
				} else if e.is_unique_violation() {
					return Err(AddAttributeError::UniqueViolation);
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
		attribute: AttributeId,
	) -> Result<AttributeInfo, GetAttributeError> {
		let mut conn = self
			.pool
			.acquire()
			.await
			.map_err(|e| GetAttributeError::DbError(Box::new(e)))?;

		let res = sqlx::query("SELECT * FROM class WHERE id=?;")
			.bind(u32::from(attribute))
			.fetch_one(&mut *conn)
			.await;

		return match res {
			Err(sqlx::Error::RowNotFound) => Err(GetAttributeError::NotFound),
			Err(e) => Err(GetAttributeError::DbError(Box::new(e))),
			Ok(res) => Ok(AttributeInfo {
				id: res.get::<u32, _>("id").into(),
				class: res.get::<u32, _>("id").into(),
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
		attribute: AttributeId,
		new_name: &str,
	) -> Result<(), RenameAttributeError> {
		match check_name(new_name) {
			Ok(()) => {}
			Err(e) => return Err(RenameAttributeError::NameError(e)),
		}

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
			Ok(_) => Ok(()),

			Err(sqlx::Error::Database(e)) => {
				if e.is_unique_violation() {
					Err(RenameAttributeError::UniqueViolation)
				} else {
					let e = Box::new(sqlx::Error::Database(e));
					Err(RenameAttributeError::DbError(e))
				}
			}

			Err(e) => Err(RenameAttributeError::DbError(Box::new(e))),
		};
	}

	async fn del_attribute(&self, attribute: AttributeId) -> Result<(), DeleteAttributeError> {
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

	//
	// MARK: Item
	//

	async fn add_item(
		&self,
		in_class: ClassId,
		attributes: BTreeMap<AttributeId, AttrData>,
	) -> Result<ItemId, AddItemError> {
		// Start transaction
		let mut conn = self
			.pool
			.acquire()
			.await
			.map_err(|e| AddItemError::DbError(Box::new(e)))?;
		let mut t = conn
			.begin()
			.await
			.map_err(|e| AddItemError::DbError(Box::new(e)))?;

		let res = sqlx::query("INSERT INTO item(class_id) VALUES ?;")
			.bind(u32::from(in_class))
			.execute(&mut *t)
			.await;

		let new_item: ItemId = match res {
			Ok(x) => u32::try_from(x.last_insert_rowid()).unwrap().into(),
			Err(sqlx::Error::Database(e)) => {
				if e.is_foreign_key_violation() {
					return Err(AddItemError::NoSuchClass);
				} else {
					let e = Box::new(sqlx::Error::Database(e));
					return Err(AddItemError::DbError(e));
				}
			}
			Err(e) => return Err(AddItemError::DbError(Box::new(e))),
		};

		for (attr, value) in &attributes {
			// Make sure this attribute comes from this class
			match sqlx::query("SELECT FROM attribute WHERE id=? AND class_id=?;")
				.fetch_one(&mut *t)
				.await
			{
				Ok(_) => {}
				Err(sqlx::Error::RowNotFound) => return Err(AddItemError::ForeignAttribute),
				Err(e) => return Err(AddItemError::DbError(Box::new(e))),
			}

			// Create the attribute instance
			let res = sqlx::query(
				"INSERT INTO attribute_instance(item_id, attribute_id, attribute_value)
				VALUES (?, ?, ?);",
			)
			.bind(u32::from(new_item))
			.bind(u32::from(*attr))
			.bind(serde_json::to_string(&value).unwrap())
			.execute(&mut *t)
			.await;

			match res {
				Ok(_) => {}
				Err(sqlx::Error::Database(e)) => {
					if e.is_foreign_key_violation() {
						return Err(AddItemError::BadAttribute);
					} else {
						let e = Box::new(sqlx::Error::Database(e));
						return Err(AddItemError::DbError(e));
					}
				}
				Err(e) => return Err(AddItemError::DbError(Box::new(e))),
			};
		}

		t.commit()
			.await
			.map_err(|e| AddItemError::DbError(Box::new(e)))?;

		return Ok(new_item);
	}

	async fn get_item(&self, item: ItemId) -> Result<ItemInfo, GetItemError> {
		unimplemented!()
	}

	async fn del_item(&self, item: ItemId) -> Result<(), DeleteItemError> {
		unimplemented!()
	}
}
