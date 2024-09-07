use crate::api::{
	data::{DatasetData, DatasetDataStub},
	errors::MetastoreError,
	handles::{AttrHandle, ClassHandle, ItemIdx},
	AttrInfo, AttributeOptions, ClassInfo, DatabaseClient, ItemData,
};
use copper_util::{mime::MimeType, names::clean_name};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use smartstring::SmartString;
use sqlx::{
	query::Query,
	sqlite::{SqliteArguments, SqliteRow},
	Connection, Row, Sqlite,
};
use std::{iter, sync::Arc};
use tracing::trace;

use super::SqliteDatabase;

#[derive(Debug, Clone, Deserialize, Serialize)]
struct BlobJsonEncoded {
	url: String,
	mime: MimeType,
}

// SQL helper functions
impl SqliteDatabase {
	pub fn get_table_name(class: ClassHandle) -> String {
		format!("class_{}", u32::from(class))
	}

	pub fn get_column_name(attr: AttrHandle) -> String {
		format!("attr_{}", u32::from(attr))
	}

	fn bind_storage<'a>(
		q: Query<'a, Sqlite, SqliteArguments<'a>>,
		storage: &'a mut DatasetData,
	) -> Result<Query<'a, Sqlite, SqliteArguments<'a>>, MetastoreError> {
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
					return Err(MetastoreError::NonNegativeViolated);
				}
				q.bind(*value)
			}

			DatasetData::Integer {
				value,
				is_non_negative,
			} => {
				if *is_non_negative && *value < 0 {
					return Err(MetastoreError::NonNegativeViolated);
				}
				q.bind(*value)
			}
		})
	}

	fn read_storage(row: &SqliteRow, attr: &AttrInfo) -> DatasetData {
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
}

impl DatabaseClient for SqliteDatabase {
	async fn add_attr(
		&self,
		class: ClassHandle,
		attr_name: &str,
		data_type: DatasetDataStub,
		options: AttributeOptions,
	) -> Result<AttrHandle, MetastoreError> {
		let attr_name = clean_name(attr_name).map_err(MetastoreError::BadAttrName)?;

		trace!(
			message = "Adding an attribute",
			attr_name = attr_name,
			data_type = ?data_type,
			options = ?options
		);

		// Start transaction
		let mut conn = self
			.pool
			.acquire()
			.await
			.map_err(|e| MetastoreError::DbError(Box::new(e)))?;
		let mut t = (conn.begin().await).map_err(|e| MetastoreError::DbError(Box::new(e)))?;

		// Get next attribute idx in this class
		let attr_idx: u32 = {
			let res =
				sqlx::query("SELECT MAX(idx) as max_idx FROM meta_attributes WHERE class_id=?;")
					.bind(u32::from(class))
					.fetch_one(&mut *t)
					.await;

			match res {
				Err(sqlx::Error::RowNotFound) => {
					return Err(MetastoreError::BadClassHandle);
				}
				Err(e) => return Err(MetastoreError::DbError(Box::new(e))),
				Ok(res) => res.get::<u32, _>("max_idx") + 1,
			}
		};

		// Add attribute metadata
		let new_attr = {
			let res = sqlx::query(
				"
				INSERT INTO meta_attributes (
					class_id, pretty_name, data_type,
					is_unique, is_not_null, idx
				) VALUES (?, ?, ?, ?, ?, ?);
				",
			)
			.bind(u32::from(class))
			.bind(&attr_name)
			.bind(serde_json::to_string(&data_type).unwrap())
			.bind(options.unique)
			.bind(false)
			//.bind(options.not_null)
			.bind(attr_idx)
			.execute(&mut *t)
			.await;

			match res {
				Err(sqlx::Error::Database(e)) => {
					if e.is_unique_violation() {
						return Err(MetastoreError::DuplicateAttrName(attr_name.into()));
					} else {
						return Err(MetastoreError::DbError(Box::new(sqlx::Error::Database(e))));
					}
				}
				Err(e) => return Err(MetastoreError::DbError(Box::new(e))),
				Ok(x) => u32::try_from(x.last_insert_rowid()).unwrap(),
			}
		};

		// Find table to modify
		let table_name = Self::get_table_name(class);
		let column_name = Self::get_column_name(new_attr.into());

		// Map internal type to sqlite type
		let data_type_str = match data_type {
			DatasetDataStub::Text => "TEXT",
			DatasetDataStub::Integer { .. } => "INTEGER",
			DatasetDataStub::Boolean => "INTEGER",
			DatasetDataStub::Float { .. } => "REAL",
			DatasetDataStub::Blob => "TEXT",
			DatasetDataStub::Reference { .. } => "INTEGER",
			DatasetDataStub::Hash { .. } => "BLOB",
		};

		//let not_null = if options.not_null { " NOT NULL" } else { "" };
		let not_null = "";

		// Add foreign key if necessary
		let references = match data_type {
			DatasetDataStub::Reference { class } => {
				format!(" REFERENCES \"{}\"(id)", Self::get_table_name(class))
			}

			DatasetDataStub::Blob => " REFERENCES meta_blobs(id)".to_string(),
			_ => "".into(),
		};

		// Add new column

		sqlx::query(&format!(
				"ALTER TABLE \"{table_name}\" ADD \"{column_name}\" {data_type_str}{not_null}{references};",
			))
		.execute(&mut *t)
		.await
		.map_err(|e| MetastoreError::DbError(Box::new(e)))?;

		// Add unique constraint if necessary
		if options.unique {
			sqlx::query(&format!(
					"CREATE UNIQUE INDEX \"unique_{table_name}_{column_name}\" ON \"{table_name}\"(\"{column_name}\");",
				))
			.execute(&mut *t)
			.await
			.map_err(|e| MetastoreError::DbError(Box::new(e)))?;
		}

		// Commit transaction
		t.commit()
			.await
			.map_err(|e| MetastoreError::DbError(Box::new(e)))?;

		// We changed our schema, so we must clear the statement cache. If we don't, sqlx
		// will panic if the cached statement query becomes out-of date.
		// (e.g, if we create/delete a db column)
		conn.clear_cached_statements()
			.await
			.map_err(|e| MetastoreError::DbError(Box::new(e)))?;

		Ok(new_attr.into())
	}

	async fn add_class(&self, class_name: &str) -> Result<ClassHandle, MetastoreError> {
		let class_name = clean_name(class_name).map_err(MetastoreError::BadClassName)?;

		trace!(message = "Adding a class", class_name);

		// Start transaction
		let mut conn = self
			.pool
			.acquire()
			.await
			.map_err(|e| MetastoreError::DbError(Box::new(e)))?;
		let mut t = conn
			.begin()
			.await
			.map_err(|e| MetastoreError::DbError(Box::new(e)))?;

		// Add metadata
		let new_class_id = {
			let res = sqlx::query("INSERT INTO meta_classes (pretty_name) VALUES (?);")
				.bind(&class_name)
				.execute(&mut *t)
				.await;

			match res {
				Err(sqlx::Error::Database(e)) => {
					if e.is_unique_violation() {
						return Err(MetastoreError::DuplicateClassName(class_name.into()));
					} else {
						return Err(MetastoreError::DbError(Box::new(sqlx::Error::Database(e))));
					}
				}
				Err(e) => return Err(MetastoreError::DbError(Box::new(e))),
				Ok(res) => u32::try_from(res.last_insert_rowid()).unwrap(),
			}
		};
		let table_name = Self::get_table_name(new_class_id.into());

		// Create new table
		sqlx::query(&format!(
			"CREATE TABLE IF NOT EXISTS \"{table_name}\" (id INTEGER PRIMARY KEY NOT NULL);"
		))
		.execute(&mut *t)
		.await
		.map_err(|e| MetastoreError::DbError(Box::new(e)))?;

		// Commit transaction
		t.commit()
			.await
			.map_err(|e| MetastoreError::DbError(Box::new(e)))?;

		// We changed our schema, so we must clear the statement cache. If we don't, sqlx
		// will panic if the cached statement query becomes out-of date.
		// (e.g, if we create/delete a db column)
		conn.clear_cached_statements()
			.await
			.map_err(|e| MetastoreError::DbError(Box::new(e)))?;

		return Ok(new_class_id.into());
	}

	async fn add_item(
		&self,
		class: ClassHandle,
		mut attrs: Vec<(AttrHandle, DatasetData)>,
	) -> Result<ItemIdx, MetastoreError> {
		// Start transaction
		let mut conn = self
			.pool
			.acquire()
			.await
			.map_err(|e| MetastoreError::DbError(Box::new(e)))?;
		let mut t = conn
			.begin()
			.await
			.map_err(|e| MetastoreError::DbError(Box::new(e)))?;

		let table_name = Self::get_table_name(class);

		trace!(
			message = "Adding an item",
			to_class = ?class,
			?attrs
		);

		// Add new row with data
		let res = if attrs.is_empty() {
			// If we were given no attributes
			sqlx::query(&format!("INSERT INTO \"{table_name}\" DEFAULT VALUES;",))
				.execute(&mut *t)
				.await
		} else {
			// Find rows of all provided attributes
			let attr_names = attrs
				.iter()
				.map(|(h, _)| Self::get_column_name(*h))
				.join(", ");

			let attr_values = iter::repeat('?').take(attrs.len()).join(", ");

			let q_str =
				format!("INSERT INTO \"{table_name}\" ({attr_names}) VALUES ({attr_values});",);
			let mut q = sqlx::query(&q_str);

			for (_, value) in &mut attrs {
				q = Self::bind_storage(q, value)?;
			}

			q.execute(&mut *t).await
		};

		// Handle errors
		let id = match res {
			Err(sqlx::Error::Database(e)) => {
				if e.is_unique_violation() {
					return Err(MetastoreError::UniqueViolated);
				} else {
					return Err(MetastoreError::DbError(Box::new(sqlx::Error::Database(e))));
				}
			}
			Err(e) => return Err(MetastoreError::DbError(Box::new(e))),
			Ok(res) => res.last_insert_rowid(),
		};

		// Commit transaction
		t.commit()
			.await
			.map_err(|e| MetastoreError::DbError(Box::new(e)))?;

		Ok(u32::try_from(id).unwrap().into())
	}

	async fn del_attr(&self, attr: AttrHandle) -> Result<(), MetastoreError> {
		// Start transaction
		let mut conn = self
			.pool
			.acquire()
			.await
			.map_err(|e| MetastoreError::DbError(Box::new(e)))?;
		let mut t = conn
			.begin()
			.await
			.map_err(|e| MetastoreError::DbError(Box::new(e)))?;

		trace!(message = "Deleting an attribute", ?attr);

		// Get this attributes' class
		let (class_id, is_unique): (ClassHandle, bool) = {
			let res = sqlx::query("SELECT class_id, is_unique FROM meta_attributes WHERE id=?;")
				.bind(u32::from(attr))
				.fetch_one(&mut *t)
				.await;

			match res {
				Err(e) => return Err(MetastoreError::DbError(Box::new(e))),
				Ok(res) => (
					res.get::<u32, _>("class_id").into(),
					res.get::<bool, _>("is_unique"),
				),
			}
		};

		// Get the table we want to modify
		let table_name = Self::get_table_name(class_id);
		let column_name = Self::get_column_name(attr);

		// Delete constraints
		// (This must be done BEFORE deleting the column)
		if is_unique {
			if let Err(e) =
				sqlx::query(&format!("DROP INDEX \"unique_{table_name}_{column_name}\""))
					.bind(u32::from(attr))
					.execute(&mut *t)
					.await
			{
				return Err(MetastoreError::DbError(Box::new(e)));
			};
		}

		// Delete attribute metadata
		if let Err(e) = sqlx::query("DELETE FROM meta_attributes WHERE id=?;")
			.bind(u32::from(attr))
			.execute(&mut *t)
			.await
		{
			return Err(MetastoreError::DbError(Box::new(e)));
		};

		// Delete attribute column
		let q_str = format!("ALTER TABLE \"{table_name}\" DROP COLUMN \"{column_name}\";");
		if let Err(e) = sqlx::query(&q_str).execute(&mut *t).await {
			return Err(MetastoreError::DbError(Box::new(e)));
		};

		// Finish
		if let Err(e) = t.commit().await {
			return Err(MetastoreError::DbError(Box::new(e)));
		};

		// We changed our schema, so we must clear the statement cache. If we don't, sqlx
		// will panic if the cached statement query becomes out-of date.
		// (e.g, if we create/delete a db column)
		conn.clear_cached_statements()
			.await
			.map_err(|e| MetastoreError::DbError(Box::new(e)))?;

		return Ok(());
	}

	async fn del_class(&self, class: ClassHandle) -> Result<(), MetastoreError> {
		// Get these FIRST, or we'll deadlock
		let attrs = self.class_get_attrs(class).await?;
		let backlinks = self.class_get_backlinks(class).await?;

		// If any other dataset has references to this class,
		// we can't delete it. Those reference attrs must first be removed.
		if backlinks.iter().any(|x| x.handle != class) {
			return Err(MetastoreError::DeleteClassDanglingRef(
				// Filter the class we tried to delete from the error vec
				backlinks
					.into_iter()
					.filter_map(|c| {
						if c.handle == class {
							None
						} else {
							Some(c.name)
						}
					})
					.collect(),
			));
		}

		trace!(message = "Deleting a class", ?class);

		// Start transaction
		let mut conn = self
			.pool
			.acquire()
			.await
			.map_err(|e| MetastoreError::DbError(Box::new(e)))?;
		let mut t = conn
			.begin()
			.await
			.map_err(|e| MetastoreError::DbError(Box::new(e)))?;

		// Get the table we want to modify
		let table_name = Self::get_table_name(class);

		// Delete all attribute metadata
		{
			// Generate query
			let q_str = format!(
				"DELETE FROM meta_attributes WHERE id IN ({});",
				iter::repeat('?').take(attrs.len()).join(", ")
			);

			// Bind each attr id
			let mut q = sqlx::query(&q_str);
			for a in attrs {
				q = q.bind(u32::from(a.handle));
			}

			// Execute query
			if let Err(e) = q.execute(&mut *t).await {
				return Err(MetastoreError::DbError(Box::new(e)));
			};
		}

		// Delete class metadata
		if let Err(e) = sqlx::query("DELETE FROM meta_classes WHERE id=?;")
			.bind(u32::from(class))
			.execute(&mut *t)
			.await
		{
			return Err(MetastoreError::DbError(Box::new(e)));
		};

		// Drop class table
		let q_str = format!("DROP TABLE \"{table_name}\";",);
		if let Err(e) = sqlx::query(&q_str).execute(&mut *t).await {
			return Err(MetastoreError::DbError(Box::new(e)));
		};

		// Finish
		if let Err(e) = t.commit().await {
			return Err(MetastoreError::DbError(Box::new(e)));
		};

		// We changed our schema, so we must clear the statement cache. If we don't, sqlx
		// will panic if the cached statement query becomes out-of date.
		// (e.g, if we create/delete a db column)
		conn.clear_cached_statements()
			.await
			.map_err(|e| MetastoreError::DbError(Box::new(e)))?;

		return Ok(());
	}

	async fn del_item(&self, _item: ItemIdx) -> Result<(), MetastoreError> {
		unimplemented!()
	}

	async fn get_attr_by_name(
		&self,
		class: ClassHandle,
		attr_name: &str,
	) -> Result<Option<AttrInfo>, MetastoreError> {
		let mut conn = self
			.pool
			.acquire()
			.await
			.map_err(|e| MetastoreError::DbError(Box::new(e)))?;

		let res = sqlx::query(
			"
				SELECT id, idx, class_id, pretty_name, data_type, idx
				FROM meta_attributes
				WHERE class_id=? AND pretty_name=?;
			",
		)
		.bind(u32::from(class))
		.bind(attr_name)
		.fetch_one(&mut *conn)
		.await;

		return match res {
			Err(sqlx::Error::RowNotFound) => Ok(None),
			Err(e) => Err(MetastoreError::DbError(Box::new(e))),
			Ok(res) => Ok(Some(AttrInfo {
				idx: res.get::<u32, _>("idx"),
				handle: res.get::<u32, _>("id").into(),
				class: res.get::<u32, _>("class_id").into(),
				name: res.get::<String, _>("pretty_name").into(),
				data_type: serde_json::from_str(res.get::<&str, _>("data_type")).unwrap(),
			})),
		};
	}

	async fn get_attr(&self, attr: AttrHandle) -> Result<AttrInfo, MetastoreError> {
		let mut conn = self
			.pool
			.acquire()
			.await
			.map_err(|e| MetastoreError::DbError(Box::new(e)))?;
		let res = sqlx::query(
			"
			SELECT id, idx, class_id, pretty_name, data_type
			FROM meta_attributes
			WHERE id=?;",
		)
		.bind(u32::from(attr))
		.fetch_one(&mut *conn)
		.await;

		return match res {
			Err(sqlx::Error::RowNotFound) => Err(MetastoreError::BadAttrHandle),
			Err(e) => Err(MetastoreError::DbError(Box::new(e))),
			Ok(res) => Ok(AttrInfo {
				idx: res.get::<u32, _>("idx"),
				handle: res.get::<u32, _>("id").into(),
				class: res.get::<u32, _>("class_id").into(),
				name: res.get::<String, _>("pretty_name").into(),
				data_type: serde_json::from_str(res.get::<&str, _>("data_type")).unwrap(),
			}),
		};
	}

	async fn get_class_by_name(
		&self,
		class_name: &str,
	) -> Result<Option<ClassInfo>, MetastoreError> {
		let mut conn = self
			.pool
			.acquire()
			.await
			.map_err(|e| MetastoreError::DbError(Box::new(e)))?;

		let res = sqlx::query("SELECT id, pretty_name FROM meta_classes WHERE pretty_name=?;")
			.bind(class_name)
			.fetch_one(&mut *conn)
			.await;

		return match res {
			Err(sqlx::Error::RowNotFound) => Ok(None),
			Err(e) => Err(MetastoreError::DbError(Box::new(e))),
			Ok(res) => Ok(Some(ClassInfo {
				handle: res.get::<u32, _>("id").into(),
				name: res.get::<&str, _>("pretty_name").into(),
			})),
		};
	}

	async fn get_class(&self, class: ClassHandle) -> Result<ClassInfo, MetastoreError> {
		let mut conn = self
			.pool
			.acquire()
			.await
			.map_err(|e| MetastoreError::DbError(Box::new(e)))?;

		let res = sqlx::query("SELECT id, pretty_name FROM meta_classes WHERE id=?;")
			.bind(u32::from(class))
			.fetch_one(&mut *conn)
			.await;

		return match res {
			Err(sqlx::Error::RowNotFound) => Err(MetastoreError::BadClassHandle),
			Err(e) => Err(MetastoreError::DbError(Box::new(e))),
			Ok(res) => Ok(ClassInfo {
				handle: res.get::<u32, _>("id").into(),
				name: res.get::<&str, _>("pretty_name").into(),
			}),
		};
	}

	async fn get_all_attrs(&self) -> Result<Vec<AttrInfo>, MetastoreError> {
		let mut conn = self
			.pool
			.acquire()
			.await
			.map_err(|e| MetastoreError::DbError(Box::new(e)))?;

		let res = sqlx::query(
			"SELECT id, idx, class_id, pretty_name, data_type FROM meta_attributes ORDER BY idx;",
		)
		.fetch_all(&mut *conn)
		.await
		.map_err(|e| MetastoreError::DbError(Box::new(e)))?;

		return Ok(res
			.into_iter()
			.map(|res| AttrInfo {
				idx: res.get::<u32, _>("idx"),
				handle: res.get::<u32, _>("id").into(),
				class: res.get::<u32, _>("class_id").into(),
				name: res.get::<String, _>("pretty_name").into(),
				data_type: serde_json::from_str(res.get::<&str, _>("data_type")).unwrap(),
			})
			.collect());
	}

	async fn get_all_classes(&self) -> Result<Vec<ClassInfo>, MetastoreError> {
		let mut conn = self
			.pool
			.acquire()
			.await
			.map_err(|e| MetastoreError::DbError(Box::new(e)))?;

		let res = sqlx::query("SELECT pretty_name, id FROM meta_classes ORDER BY id;")
			.fetch_all(&mut *conn)
			.await
			.map_err(|e| MetastoreError::DbError(Box::new(e)))?;

		return Ok(res
			.into_iter()
			.map(|x| ClassInfo {
				handle: x.get::<u32, _>("id").into(),
				name: x.get::<String, _>("pretty_name").into(),
			})
			.collect());
	}

	async fn class_set_name(&self, class: ClassHandle, name: &str) -> Result<(), MetastoreError> {
		let name = clean_name(name).map_err(MetastoreError::BadClassName)?;

		// Make sure this name isn't already taken
		let x = self.get_class_by_name(&name).await?;
		if x.is_some() {
			return Err(MetastoreError::DuplicateClassName(name.into()));
		}

		let mut conn = self
			.pool
			.acquire()
			.await
			.map_err(|e| MetastoreError::DbError(Box::new(e)))?;

		let res = sqlx::query("UPDATE meta_classes SET pretty_name=? WHERE id=?;")
			.bind(name)
			.bind(u32::from(class))
			.execute(&mut *conn)
			.await;

		match res {
			Err(sqlx::Error::RowNotFound) => return Err(MetastoreError::BadClassHandle),
			Err(e) => return Err(MetastoreError::DbError(Box::new(e))),
			_ => {}
		};

		return Ok(());
	}

	async fn class_num_attrs(&self, _class: ClassHandle) -> Result<usize, MetastoreError> {
		unimplemented!()
	}

	async fn class_get_attrs(&self, class: ClassHandle) -> Result<Vec<AttrInfo>, MetastoreError> {
		let mut conn = self
			.pool
			.acquire()
			.await
			.map_err(|e| MetastoreError::DbError(Box::new(e)))?;

		let res = sqlx::query(
			"
			SELECT id, idx, pretty_name, data_type, class_id
			FROM meta_attributes WHERE class_id=?
			ORDER BY idx;
			",
		)
		.bind(u32::from(class))
		.fetch_all(&mut *conn)
		.await;

		let res = match res {
			Err(sqlx::Error::RowNotFound) => return Err(MetastoreError::BadClassHandle),
			Err(e) => return Err(MetastoreError::DbError(Box::new(e))),
			Ok(res) => res,
		};

		Ok(res
			.into_iter()
			.map(|x| AttrInfo {
				idx: x.get::<u32, _>("idx"),
				handle: x.get::<u32, _>("id").into(),
				class: x.get::<u32, _>("class_id").into(),
				name: x.get::<String, _>("pretty_name").into(),
				data_type: serde_json::from_str(x.get::<&str, _>("data_type")).unwrap(),
			})
			.collect())
	}

	async fn class_get_backlinks(
		&self,
		class: ClassHandle,
	) -> Result<Vec<ClassInfo>, MetastoreError> {
		let classes = self.get_all_classes().await?;
		let mut out = Vec::new();
		for i_class in classes {
			for attr in self.class_get_attrs(i_class.handle).await? {
				if let DatasetDataStub::Reference { class: ref_class } = attr.data_type {
					if class == ref_class {
						out.push(ClassInfo {
							handle: i_class.handle,
							name: i_class.name,
						});
						// We include each class exactly once, so break here.
						break;
					}
				}
			}
		}

		return Ok(out);
	}

	async fn attr_set_name(&self, attr: AttrHandle, name: &str) -> Result<(), MetastoreError> {
		let name = clean_name(name).map_err(MetastoreError::BadAttrName)?;

		// Make sure this name isn't already taken
		let x = self.get_attr(attr).await?;
		let x = self.get_attr_by_name(x.class, &name).await?;
		if x.is_some() {
			return Err(MetastoreError::DuplicateAttrName(name.into()));
		}

		let mut conn = self
			.pool
			.acquire()
			.await
			.map_err(|e| MetastoreError::DbError(Box::new(e)))?;

		let res = sqlx::query("UPDATE meta_attributes SET pretty_name=? WHERE id=?;")
			.bind(name)
			.bind(u32::from(attr))
			.execute(&mut *conn)
			.await;

		match res {
			Err(sqlx::Error::RowNotFound) => return Err(MetastoreError::BadAttrHandle),
			Err(e) => return Err(MetastoreError::DbError(Box::new(e))),
			_ => {}
		};

		return Ok(());
	}

	async fn find_item_with_attr(
		&self,
		attr: AttrHandle,
		mut attr_value: DatasetData,
	) -> Result<Vec<ItemIdx>, MetastoreError> {
		let mut conn = self
			.pool
			.acquire()
			.await
			.map_err(|e| MetastoreError::DbError(Box::new(e)))?;

		// Find table and column to search
		let column_name = Self::get_column_name(attr);
		let table_name: String = {
			let res = sqlx::query(
				"
					SELECT meta_classes.id AS class_id
					FROM meta_attributes
					INNER JOIN meta_classes ON meta_classes.id = meta_attributes.class_id
					WHERE meta_attributes.id=?;
					",
			)
			.bind(u32::from(attr))
			.fetch_one(&mut *conn)
			.await;

			match res {
				Err(sqlx::Error::RowNotFound) => Err(MetastoreError::BadAttrHandle),
				Err(e) => Err(MetastoreError::DbError(Box::new(e))),
				Ok(res) => {
					let class_id: ClassHandle = res.get::<u32, _>("class_id").into();
					Ok(Self::get_table_name(class_id))
				}
			}
		}?;

		let query_str = format!("SELECT id FROM \"{table_name}\" WHERE \"{column_name}\"=?;");
		let mut q = sqlx::query(&query_str);
		q = Self::bind_storage(q, &mut attr_value)?;

		let res = q.bind(u32::from(attr)).fetch_all(&mut *conn).await;
		return match res {
			Err(sqlx::Error::RowNotFound) => Ok(Vec::new()),
			Err(e) => Err(MetastoreError::DbError(Box::new(e))),
			Ok(res) => Ok(res
				.iter()
				.map(|row| row.get::<u32, _>("id").into())
				.collect()),
		};
	}

	async fn get_items(
		&self,
		class: ClassHandle,
		page_size: u32,
		start_at: u32,
	) -> Result<Vec<ItemData>, MetastoreError> {
		// Do this first, prevent deadlock
		let attrs = self.class_get_attrs(class).await?;
		let table_name = Self::get_table_name(class);
		let mut conn = self
			.pool
			.acquire()
			.await
			.map_err(|e| MetastoreError::DbError(Box::new(e)))?;

		let res = sqlx::query(&format!(
				"SELECT * FROM \"{table_name}\" ORDER BY id LIMIT \"{page_size}\" OFFSET \"{start_at}\" ;"
			))
		.fetch_all(&mut *conn)
		.await
		.map_err(|e| MetastoreError::DbError(Box::new(e)))?;

		let mut out = Vec::new();
		for row in res {
			out.push(ItemData {
				handle: row.get::<u32, _>("id").into(),
				attrs: attrs
					.iter()
					.map(|attr| (attr.handle, Self::read_storage(&row, attr)))
					.collect(),
			})
		}

		return Ok(out);
	}

	async fn count_items(&self, class: ClassHandle) -> Result<u32, MetastoreError> {
		let table_name = Self::get_table_name(class);
		let mut conn = self
			.pool
			.acquire()
			.await
			.map_err(|e| MetastoreError::DbError(Box::new(e)))?;

		let res = sqlx::query(&format!(
			"SELECT COUNT(1) as \"count\" FROM \"{table_name}\";"
		))
		.fetch_one(&mut *conn)
		.await
		.map_err(|e| MetastoreError::DbError(Box::new(e)))?;

		return Ok(res.get("count"));
	}

	async fn get_item(
		&self,
		class: ClassHandle,
		item: ItemIdx,
	) -> Result<ItemData, MetastoreError> {
		// Do this first, prevent deadlock
		let attrs = self.class_get_attrs(class).await?;
		let table_name = Self::get_table_name(class);
		let mut conn = self
			.pool
			.acquire()
			.await
			.map_err(|e| MetastoreError::DbError(Box::new(e)))?;

		let res = sqlx::query(&format!("SELECT * FROM \"{table_name}\" WHERE id=?;"))
			.bind(u32::from(item))
			.fetch_one(&mut *conn)
			.await
			.map_err(|e| match e {
				sqlx::Error::RowNotFound => MetastoreError::BadItemIdx,
				_ => MetastoreError::DbError(Box::new(e)),
			})?;

		let out = ItemData {
			handle: res.get::<u32, _>("id").into(),
			attrs: attrs
				.iter()
				.map(|attr| (attr.handle, Self::read_storage(&res, attr)))
				.collect(),
		};

		return Ok(out);
	}

	async fn get_item_attr(
		&self,
		attr: AttrHandle,
		item: ItemIdx,
	) -> Result<DatasetData, MetastoreError> {
		// Do this first, prevent deadlock
		let attr_data = self.get_attr(attr).await?;
		let table_name = Self::get_table_name(attr_data.class);
		let column_name = Self::get_column_name(attr_data.handle);
		let mut conn = self
			.pool
			.acquire()
			.await
			.map_err(|e| MetastoreError::DbError(Box::new(e)))?;

		let res = sqlx::query(&format!(
			"SELECT \"{column_name}\" FROM \"{table_name}\" WHERE id=?;"
		))
		.bind(u32::from(item))
		.fetch_one(&mut *conn)
		.await;

		return match res {
			Err(sqlx::Error::RowNotFound) => Err(MetastoreError::BadItemIdx),
			Err(e) => Err(MetastoreError::DbError(Box::new(e))),
			Ok(row) => Ok(Self::read_storage(&row, &attr_data)),
		};
	}
}
