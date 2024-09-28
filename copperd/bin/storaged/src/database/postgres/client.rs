use async_trait::async_trait;
use copper_edged::UserId;
use copper_storaged::{
	AttrData, AttrDataStub, AttributeId, AttributeInfo, ClassId, ClassInfo, DatasetId, DatasetInfo,
	ResultOrDirect, Transaction, TransactionAction,
};
use copper_util::{names::check_name, MimeType};
use serde::{Deserialize, Serialize};
use sqlx::{Connection, Row};

use super::{helpers, PgDatabaseClient};
use crate::database::base::{
	client::{AttributeOptions, DatabaseClient},
	errors::{
		attribute::{
			AddAttributeError, DeleteAttributeError, GetAttributeError, RenameAttributeError,
		},
		class::{AddClassError, DeleteClassError, GetClassError, RenameClassError},
		dataset::{AddDatasetError, DeleteDatasetError, GetDatasetError, RenameDatasetError},
		transaction::ApplyTransactionError,
	},
};

#[derive(Debug, Clone, Deserialize, Serialize)]
struct BlobJsonEncoded {
	url: String,
	mime: MimeType,
}

#[async_trait]
impl DatabaseClient for PgDatabaseClient {
	//
	// MARK: Dataset
	//

	async fn add_dataset(&self, name: &str, user: UserId) -> Result<DatasetId, AddDatasetError> {
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

		let res =
			sqlx::query("INSERT INTO dataset (pretty_name, owner) VALUES ($1, $2) RETURNING id;")
				.bind(name)
				.bind(i64::from(user))
				.fetch_one(&mut *t)
				.await;

		t.commit()
			.await
			.map_err(|e| AddDatasetError::DbError(Box::new(e)))?;

		let new_handle: DatasetId = match res {
			Ok(res) => res.get::<i64, _>("id").into(),
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

		let classes: Vec<ClassInfo> = {
			let res = sqlx::query("SELECT * FROM class WHERE dataset_id=$1;")
				.bind(i64::from(dataset))
				.fetch_all(&mut *conn)
				.await;

			match res {
				Err(e) => return Err(GetDatasetError::DbError(Box::new(e))),
				Ok(rows) => {
					let mut classes = Vec::new();

					for r in rows {
						let class_id: ClassId = r.get::<i64, _>("id").into();

						let res = sqlx::query("SELECT * FROM attribute WHERE class_id=$1;")
							.bind(i64::from(class_id))
							.fetch_all(&mut *conn)
							.await;

						let attributes: Vec<AttributeInfo> = match res {
							Err(e) => return Err(GetDatasetError::DbError(Box::new(e))),
							Ok(rows) => rows
								.into_iter()
								.map(|row| AttributeInfo {
									id: row.get::<i64, _>("id").into(),
									class: row.get::<i64, _>("id").into(),
									order: row.get::<i32, _>("attr_order"),
									name: row.get::<String, _>("pretty_name").into(),
									data_type: serde_json::from_str(
										row.get::<&str, _>("data_type"),
									)
									.unwrap(),
									is_unique: row.get("is_unique"),
									is_not_null: row.get("is_not_null"),
								})
								.collect(),
						};

						classes.push(ClassInfo {
							id: class_id,
							name: r.get::<String, _>("pretty_name").into(),
							attributes,
						});
					}

					classes
				}
			}
		};

		let res = sqlx::query("SELECT * FROM dataset WHERE id=$1;")
			.bind(i64::from(dataset))
			.fetch_one(&mut *conn)
			.await;

		return match res {
			Err(sqlx::Error::RowNotFound) => Err(GetDatasetError::NotFound),
			Err(e) => Err(GetDatasetError::DbError(Box::new(e))),
			Ok(res) => Ok(DatasetInfo {
				id: res.get::<i64, _>("id").into(),
				owner: res.get::<i64, _>("owner").into(),
				name: res.get::<String, _>("pretty_name").into(),
				classes,
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

		let res = sqlx::query("UPDATE dataset SET pretty_name=$1 WHERE id=$2;")
			.bind(new_name)
			.bind(i64::from(dataset))
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
		sqlx::query("DELETE FROM dataset WHERE id=$1;")
			.bind(i64::from(dataset))
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

		let res = sqlx::query(
			"INSERT INTO class (dataset_id, pretty_name) VALUES ($1, $2) RETURNING id;",
		)
		.bind(i64::from(in_dataset))
		.bind(name)
		.fetch_one(&mut *t)
		.await;

		t.commit()
			.await
			.map_err(|e| AddClassError::DbError(Box::new(e)))?;

		let new_handle: ClassId = match res {
			Ok(res) => res.get::<i64, _>("id").into(),
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

		let res = sqlx::query("SELECT * FROM attribute WHERE class_id=$1;")
			.bind(i64::from(class))
			.fetch_all(&mut *conn)
			.await;

		let attributes: Vec<AttributeInfo> = match res {
			Err(e) => return Err(GetClassError::DbError(Box::new(e))),
			Ok(rows) => rows
				.into_iter()
				.map(|row| AttributeInfo {
					id: row.get::<i64, _>("id").into(),
					class: row.get::<i64, _>("id").into(),
					order: row.get::<i32, _>("attr_order"),
					name: row.get::<String, _>("pretty_name").into(),
					data_type: serde_json::from_str(row.get::<&str, _>("data_type")).unwrap(),
					is_unique: row.get("is_unique"),
					is_not_null: row.get("is_not_null"),
				})
				.collect(),
		};

		let res = sqlx::query("SELECT * FROM class WHERE id=$1;")
			.bind(i64::from(class))
			.fetch_one(&mut *conn)
			.await;

		return match res {
			Err(sqlx::Error::RowNotFound) => Err(GetClassError::NotFound),
			Err(e) => Err(GetClassError::DbError(Box::new(e))),
			Ok(res) => Ok(ClassInfo {
				id: res.get::<i64, _>("id").into(),
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

		let res = sqlx::query("UPDATE class SET pretty_name=$1 WHERE id=$2;")
			.bind(new_name)
			.bind(i64::from(class))
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
		sqlx::query("DELETE FROM class WHERE id=$1;")
			.bind(i64::from(class))
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
			SELECT $1, COALESCE(MAX(attr_order) + 1 , 0), $2, $3, $4, $5 FROM attribute WHERE class_id=$6
			RETURNING id;",
		)
		.bind(i64::from(in_class))
		.bind(name)
		.bind(serde_json::to_string(&with_type).unwrap())
		.bind(options.unique)
		.bind(options.is_not_null)
		.bind(i64::from(in_class))
		.fetch_one(&mut *t)
		.await;

		t.commit()
			.await
			.map_err(|e| AddAttributeError::DbError(Box::new(e)))?;

		let new_handle: AttributeId = match res {
			Ok(res) => res.get::<i64, _>("id").into(),
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

		let res = sqlx::query("SELECT * FROM class WHERE id=$1;")
			.bind(i64::from(attribute))
			.fetch_one(&mut *conn)
			.await;

		return match res {
			Err(sqlx::Error::RowNotFound) => Err(GetAttributeError::NotFound),
			Err(e) => Err(GetAttributeError::DbError(Box::new(e))),
			Ok(res) => Ok(AttributeInfo {
				id: res.get::<i64, _>("id").into(),
				class: res.get::<i64, _>("id").into(),
				order: res.get::<i32, _>("attr_order"),
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

		let res = sqlx::query("UPDATE attribute SET pretty_name=$1 WHERE id=$2;")
			.bind(new_name)
			.bind(i64::from(attribute))
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
		sqlx::query("DELETE FROM attribute WHERE id=$1;")
			.bind(i64::from(attribute))
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

		let mut transaction_results: Vec<Option<AttrData>> = Vec::new();

		for action in transaction {
			match action {
				TransactionAction::AddItem {
					to_class,
					attributes,
				} => {
					let mut resolved_attributes = Vec::new();

					// Resolve references to previous actions' results
					for (k, v) in attributes {
						let value = match v {
							ResultOrDirect::Direct { value } => value,
							ResultOrDirect::Result {
								action_idx,
								expected_type,
							} => match transaction_results.get(action_idx) {
								None => return Err(ApplyTransactionError::ReferencedBadAction),
								Some(None) => {
									return Err(ApplyTransactionError::ReferencedNoneResult)
								}
								Some(Some(x)) => {
									if x.as_stub() != expected_type {
										return Err(
											ApplyTransactionError::ReferencedResultWithBadType,
										);
									}
									x.clone()
								}
							},
						};

						resolved_attributes.push((k, value));
					}

					let res = helpers::add_item(&mut t, to_class, resolved_attributes).await?;
					transaction_results.push(Some(AttrData::Reference {
						class: to_class,
						item: res,
					}))
				}
			};
		}

		t.commit()
			.await
			.map_err(|e| ApplyTransactionError::DbError(Box::new(e)))?;

		return Ok(());
	}
}
