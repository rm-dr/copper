use async_trait::async_trait;
use copper_storaged::{
	AttrDataStub, AttributeId, AttributeInfo, ClassId, ClassInfo, DatasetId, DatasetInfo,
	Transaction, TransactionAction,
};
use copper_util::MimeType;
use serde::{Deserialize, Serialize};
use sqlx::{Connection, Row};

use super::{helpers, SqliteDatabaseClient};
use crate::{
	database::base::{
		client::{AttributeOptions, DatabaseClient},
		errors::{
			attribute::{
				AddAttributeError, DeleteAttributeError, GetAttributeError, RenameAttributeError,
			},
			class::{AddClassError, DeleteClassError, GetClassError, RenameClassError},
			dataset::{AddDatasetError, DeleteDatasetError, GetDatasetError, RenameDatasetError},
			transaction::ApplyTransactionError,
		},
	},
	util::names::check_name,
};

#[derive(Debug, Clone, Deserialize, Serialize)]
struct BlobJsonEncoded {
	url: String,
	mime: MimeType,
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

		let res = sqlx::query("SELECT * FROM attribute WHERE class_id=?;")
			.bind(u32::from(class))
			.fetch_all(&mut *conn)
			.await;

		let attributes: Vec<AttributeInfo> = match res {
			Err(e) => return Err(GetClassError::DbError(Box::new(e))),
			Ok(rows) => rows
				.into_iter()
				.map(|row| AttributeInfo {
					id: row.get::<u32, _>("id").into(),
					class: row.get::<u32, _>("id").into(),
					order: row.get::<u32, _>("attr_order"),
					name: row.get::<String, _>("pretty_name").into(),
					data_type: serde_json::from_str(row.get::<&str, _>("data_type")).unwrap(),
					is_unique: row.get("is_unique"),
					is_not_null: row.get("is_not_null"),
				})
				.collect(),
		};

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
				attributes,
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
				order: res.get::<u32, _>("attr_order"),
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
	// MARK: Transaction
	//

	async fn apply_transaction(
		&self,
		transaction: Transaction,
	) -> Result<(), ApplyTransactionError> {
		// Start transaction
		let mut conn = self
			.pool
			.acquire()
			.await
			.map_err(|e| ApplyTransactionError::DbError(Box::new(e)))?;
		let mut t = conn
			.begin()
			.await
			.map_err(|e| ApplyTransactionError::DbError(Box::new(e)))?;

		for action in transaction.actions {
			match action {
				TransactionAction::AddItem {
					to_class,
					attributes,
				} => helpers::add_item(&mut t, to_class, attributes).await?,
			};
		}

		t.commit()
			.await
			.map_err(|e| ApplyTransactionError::DbError(Box::new(e)))?;

		return Ok(());
	}
}
