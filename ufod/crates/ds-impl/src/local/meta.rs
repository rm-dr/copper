use futures::executor::block_on;
use itertools::Itertools;
use smartstring::{LazyCompact, SmartString};
use sqlx::{query::Query, sqlite::SqliteArguments, Connection, Row, Sqlite};
use std::iter;
use tracing::debug;
use ufo_ds_core::{
	api::{
		blob::{BlobHandle, Blobstore},
		meta::{AttributeOptions, Metastore},
	},
	data::{MetastoreData, MetastoreDataStub},
	errors::MetastoreError,
	handles::{AttrHandle, ClassHandle, ItemHandle},
};

use super::LocalDataset;

// SQL helper functions
impl LocalDataset {
	pub fn get_table_name(class: ClassHandle) -> String {
		format!("class_{}", u32::from(class))
	}

	pub fn get_column_name(attr: AttrHandle) -> String {
		format!("attr_{}", u32::from(attr))
	}

	fn bind_storage<'a>(
		q: Query<'a, Sqlite, SqliteArguments<'a>>,
		storage: &'a mut MetastoreData,
	) -> Query<'a, Sqlite, SqliteArguments<'a>> {
		match storage {
			// We MUST bind something, even for null values.
			// If we don't, the null value's '?' won't be used
			// and all following fields will be shifted left.
			MetastoreData::None(_) => q.bind(None::<u32>),
			MetastoreData::Text(s) => q.bind(&**s),
			MetastoreData::Path(p) => q.bind(p.to_str().unwrap()),
			MetastoreData::Integer(x) => q.bind(&*x),
			MetastoreData::PositiveInteger(x) => q.bind(i64::from_be_bytes(x.to_be_bytes())),
			MetastoreData::Boolean(x) => q.bind(*x),
			MetastoreData::Float(x) => q.bind(&*x),
			MetastoreData::Hash { data, .. } => q.bind(&**data),
			MetastoreData::Binary { data, .. } => q.bind(&**data),
			MetastoreData::Reference { item, .. } => q.bind(u32::from(*item)),
			MetastoreData::Blob { handle } => q.bind(u32::from(*handle)),
		}
	}

	/// Delete all blobs that are not referenced by an attribute
	fn delete_dead_blobs(&self) -> Result<(), MetastoreError> {
		let attrs = self.get_all_attrs()?;
		let mut all_blobs = self
			.all_blobs()
			.map_err(|e| MetastoreError::BlobstoreError(e))?;

		// Do this after getting attrs to prevent deadlock
		let mut conn = self.conn.lock().unwrap();

		// Get all used blobs
		for (c_handle, a_handle, _, attr_type) in attrs {
			match attr_type {
				MetastoreDataStub::Blob => {
					let table_name = Self::get_table_name(c_handle);
					let column_name = Self::get_column_name(a_handle);

					let res = block_on(
						sqlx::query(&format!(
							"SELECT \"{column_name}\" FROM \"{table_name}\" ORDER BY id;"
						))
						.fetch_all(&mut *conn),
					)
					.map_err(|e| MetastoreError::DbError(Box::new(e)))?;

					// Remove used blobs from all_blobs
					for r in res {
						let blob_handle: BlobHandle = r.get::<u32, _>(column_name.as_str()).into();
						if let Some(index) = all_blobs.iter().position(|x| *x == blob_handle) {
							all_blobs.swap_remove(index);
						};
					}
				}
				_ => continue,
			}
		}

		// Prevent deadlock
		drop(conn);

		for b in all_blobs {
			debug!(
				message = "Deleting dead blob",
				blob_handle = ?b
			);
			self.delete_blob(b)
				.map_err(|e| MetastoreError::BlobstoreError(e))?;
		}

		return Ok(());
	}
}

impl Metastore for LocalDataset {
	fn add_attr(
		&self,
		class: ClassHandle,
		attr_name: &str,
		data_type: MetastoreDataStub,
		options: AttributeOptions,
	) -> Result<AttrHandle, MetastoreError> {
		// Start transaction
		let mut conn_lock = self.conn.lock().unwrap();
		let mut t =
			block_on(conn_lock.begin()).map_err(|e| MetastoreError::DbError(Box::new(e)))?;

		// Add attribute metadata
		let new_attr = {
			let res = block_on(
				sqlx::query(
					"
				INSERT INTO meta_attributes (
					class_id, pretty_name, data_type,
					is_unique, is_not_null
				) VALUES (?, ?, ?, ?, ?);
				",
				)
				.bind(u32::from(class))
				.bind(attr_name)
				.bind(serde_json::to_string(&data_type).unwrap())
				.bind(options.unique)
				.bind(false)
				//.bind(options.not_null)
				.execute(&mut *t),
			);

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
			MetastoreDataStub::Text => "TEXT",
			MetastoreDataStub::Integer => "INTEGER",
			MetastoreDataStub::PositiveInteger => "INTEGER",
			MetastoreDataStub::Boolean => "INTEGER",
			MetastoreDataStub::Float => "REAL",
			MetastoreDataStub::Binary => "BLOB",
			MetastoreDataStub::Blob => "INTEGER",
			MetastoreDataStub::Reference { .. } => "INTEGER",
			MetastoreDataStub::Hash { .. } => "BLOB",
		};

		//let not_null = if options.not_null { " NOT NULL" } else { "" };
		let not_null = "";

		// Add foreign key if necessary
		let references = match data_type {
			MetastoreDataStub::Reference { class } => {
				let id: u32 = {
					let res = block_on(
						sqlx::query("SELECT id FROM meta_classes WHERE id=?;")
							.bind(u32::from(class))
							.fetch_one(&mut *t),
					)
					.map_err(|e| MetastoreError::DbError(Box::new(e)))?;
					res.get("id")
				};
				format!(" REFERENCES \"{}\"(id)", Self::get_table_name(id.into()))
			}

			MetastoreDataStub::Blob => {
				format!(" REFERENCES meta_blobs(id)")
			}
			_ => "".into(),
		};

		// Add new column
		block_on(
			sqlx::query(&format!(
				"ALTER TABLE \"{table_name}\" ADD \"{column_name}\" {data_type_str}{not_null}{references};",
			))
			.execute(&mut *t),
		)
		.map_err(|e| MetastoreError::DbError(Box::new(e)))?;

		// Add unique constraint if necessary
		if options.unique {
			block_on(
				sqlx::query(&format!(
					"CREATE UNIQUE INDEX \"unique_{table_name}_{column_name}\" ON \"{table_name}\"(\"{column_name}\");",
				))
				.execute(&mut *t),
			)
			.map_err(|e| MetastoreError::DbError(Box::new(e)))?;
		}

		// Commit transaction
		block_on(t.commit()).map_err(|e| MetastoreError::DbError(Box::new(e)))?;

		Ok(u32::try_from(new_attr).unwrap().into())
	}

	fn add_class(&self, class_name: &str) -> Result<ClassHandle, MetastoreError> {
		// Start transaction
		let mut conn_lock = self.conn.lock().unwrap();
		let mut t =
			block_on(conn_lock.begin()).map_err(|e| MetastoreError::DbError(Box::new(e)))?;

		// Add metadata
		let new_class_id = {
			let res = block_on(
				sqlx::query("INSERT INTO meta_classes (pretty_name) VALUES (?);")
					.bind(class_name)
					.execute(&mut *t),
			);

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
		block_on(
			sqlx::query(&format!(
				"CREATE TABLE IF NOT EXISTS \"{table_name}\" (id INTEGER PRIMARY KEY NOT NULL);"
			))
			.execute(&mut *t),
		)
		.map_err(|e| MetastoreError::DbError(Box::new(e)))?;

		// Commit transaction
		block_on(t.commit()).map_err(|e| MetastoreError::DbError(Box::new(e)))?;

		return Ok(u32::try_from(new_class_id).unwrap().into());
	}

	fn add_item(
		&self,
		class: ClassHandle,
		mut attrs: Vec<(AttrHandle, MetastoreData)>,
	) -> Result<ItemHandle, MetastoreError> {
		// Start transaction
		let mut conn_lock = self.conn.lock().unwrap();
		let mut t =
			block_on(conn_lock.begin()).map_err(|e| MetastoreError::DbError(Box::new(e)))?;

		let table_name = Self::get_table_name(class);

		// Add new row with data
		let res = if attrs.is_empty() {
			// If we were given no attributes
			block_on(
				sqlx::query(&format!("INSERT INTO \"{table_name}\" DEFAULT VALUES;",))
					.execute(&mut *t),
			)
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
				q = Self::bind_storage(q, value);
			}

			block_on(q.execute(&mut *t))
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
		block_on(t.commit()).map_err(|e| MetastoreError::DbError(Box::new(e)))?;

		Ok(u32::try_from(id).unwrap().into())
	}

	fn del_attr(&self, attr: AttrHandle) -> Result<(), MetastoreError> {
		// Start transaction
		let mut conn_lock = self.conn.lock().unwrap();
		let mut t =
			block_on(conn_lock.begin()).map_err(|e| MetastoreError::DbError(Box::new(e)))?;

		// Get this attributes' class
		let class_id: ClassHandle = {
			let res = block_on(
				sqlx::query("SELECT class_id FROM meta_attributes WHERE id=?;")
					.bind(u32::from(attr))
					.fetch_one(&mut *t),
			);

			match res {
				Err(e) => return Err(MetastoreError::DbError(Box::new(e))),
				Ok(res) => res.get::<u32, _>("class_id").into(),
			}
		};

		// Get the table we want to modify
		let table_name = Self::get_table_name(class_id);
		let col_name = Self::get_column_name(attr);

		// Delete attribute metadata
		if let Err(e) = block_on(
			sqlx::query("DELETE FROM meta_attributes WHERE id=?;")
				.bind(u32::from(attr))
				.execute(&mut *t),
		) {
			return Err(MetastoreError::DbError(Box::new(e)));
		};

		// Delete attribute column
		let q_str = format!("ALTER TABLE \"{table_name}\" DROP COLUMN \"{col_name}\";");
		if let Err(e) = block_on(sqlx::query(&q_str).execute(&mut *t)) {
			return Err(MetastoreError::DbError(Box::new(e)));
		};

		// Finish
		if let Err(e) = block_on(t.commit()) {
			return Err(MetastoreError::DbError(Box::new(e)));
		};

		// Clean up dangling blobs
		// This locks our connection, so we must drop our existing lock first.
		drop(conn_lock);
		self.delete_dead_blobs()?;

		return Ok(());
	}

	fn del_class(&self, class: ClassHandle) -> Result<(), MetastoreError> {
		// Get these FIRST, or we'll deadlock
		let attrs = self.class_get_attrs(class)?;
		let backlinks = self.class_get_backlinks(class)?;

		// If any other dataset has references to this class,
		// we can't delete it. Those reference attrs must first be removed.
		if backlinks.iter().any(|x| x.0 != class) {
			return Err(MetastoreError::DeleteClassDanglingRef(
				// Filter the class we tried to delete from the error vec
				backlinks
					.into_iter()
					.filter_map(|(id, name)| if id == class { None } else { Some(name) })
					.collect(),
			));
		}

		// Start transaction
		let mut conn_lock = self.conn.lock().unwrap();
		let mut t =
			block_on(conn_lock.begin()).map_err(|e| MetastoreError::DbError(Box::new(e)))?;

		// TODO: check references
		// TODO: check pipelines (or don't, just mark as invalid)
		// TODO: delete blobs (here, del attr, and del item)

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
			for (attr, _, _) in attrs {
				q = q.bind(u32::from(attr));
			}

			// Execute query
			if let Err(e) = block_on(q.execute(&mut *t)) {
				return Err(MetastoreError::DbError(Box::new(e)));
			};
		}

		// Delete class metadata
		if let Err(e) = block_on(
			sqlx::query("DELETE FROM meta_classes WHERE id=?;")
				.bind(u32::from(class))
				.execute(&mut *t),
		) {
			return Err(MetastoreError::DbError(Box::new(e)));
		};

		// Drop class table
		let q_str = format!("DROP TABLE \"{table_name}\";",);
		if let Err(e) = block_on(sqlx::query(&q_str).execute(&mut *t)) {
			return Err(MetastoreError::DbError(Box::new(e)));
		};

		// Finish
		if let Err(e) = block_on(t.commit()) {
			return Err(MetastoreError::DbError(Box::new(e)));
		};

		// Clean up dangling blobs
		// This locks our connection, so we must drop our existing lock first.
		drop(conn_lock);
		self.delete_dead_blobs()?;

		return Ok(());
	}

	fn del_item(&self, _item: ItemHandle) -> Result<(), MetastoreError> {
		unimplemented!()
	}

	fn get_attr(
		&self,
		class: ClassHandle,
		attr_name: &str,
	) -> Result<Option<AttrHandle>, MetastoreError> {
		let mut conn = self.conn.lock().unwrap();

		let res = block_on(
			sqlx::query("SELECT id FROM meta_attributes WHERE class_id=? AND pretty_name=?;")
				.bind(u32::from(class))
				.bind(attr_name)
				.fetch_one(&mut *conn),
		);

		return match res {
			Err(sqlx::Error::RowNotFound) => Ok(None),
			Err(e) => Err(MetastoreError::DbError(Box::new(e))),
			Ok(res) => Ok(Some(res.get::<u32, _>("id").into())),
		};
	}

	fn get_class(&self, class_name: &str) -> Result<Option<ClassHandle>, MetastoreError> {
		let mut conn = self.conn.lock().unwrap();

		let res = block_on(
			sqlx::query("SELECT id FROM meta_classes WHERE pretty_name=?;")
				.bind(class_name)
				.fetch_one(&mut *conn),
		);

		return match res {
			Err(sqlx::Error::RowNotFound) => Ok(None),
			Err(e) => Err(MetastoreError::DbError(Box::new(e))),
			Ok(res) => Ok(Some(res.get::<u32, _>("id").into())),
		};
	}

	fn get_all_attrs(
		&self,
	) -> Result<
		Vec<(
			ClassHandle,
			AttrHandle,
			SmartString<LazyCompact>,
			MetastoreDataStub,
		)>,
		MetastoreError,
	> {
		let mut conn = self.conn.lock().unwrap();

		let res = block_on(
			sqlx::query(
				"SELECT id, class_id, pretty_name, data_type FROM meta_attributes ORDER BY id;",
			)
			.fetch_all(&mut *conn),
		)
		.map_err(|e| MetastoreError::DbError(Box::new(e)))?;

		return Ok(res
			.into_iter()
			.map(|x| {
				(
					x.get::<u32, _>("id").into(),
					x.get::<u32, _>("class_id").into(),
					x.get::<String, _>("pretty_name").into(),
					serde_json::from_str(x.get::<&str, _>("data_type")).unwrap(),
				)
			})
			.collect());
	}
	fn get_all_classes(
		&self,
	) -> Result<Vec<(ClassHandle, SmartString<LazyCompact>)>, MetastoreError> {
		let mut conn = self.conn.lock().unwrap();

		let res = block_on(
			sqlx::query("SELECT pretty_name, id FROM meta_classes ORDER BY id;")
				.fetch_all(&mut *conn),
		)
		.map_err(|e| MetastoreError::DbError(Box::new(e)))?;

		return Ok(res
			.into_iter()
			.map(|x| {
				(
					x.get::<u32, _>("id").into(),
					x.get::<String, _>("pretty_name").into(),
				)
			})
			.collect());
	}

	fn get_all_items(&self) -> Result<Vec<(ItemHandle, SmartString<LazyCompact>)>, MetastoreError> {
		unimplemented!()
	}

	fn item_get_attr(
		&self,
		_item: ItemHandle,
		_attr: AttrHandle,
	) -> Result<MetastoreData, MetastoreError> {
		unimplemented!()
	}

	fn item_get_class(&self, _item: ItemHandle) -> Result<ClassHandle, MetastoreError> {
		unimplemented!()
	}

	fn item_set_attr(
		&self,
		attr: AttrHandle,
		mut data: MetastoreData,
	) -> Result<(), MetastoreError> {
		// Start transaction
		let mut conn_lock = self.conn.lock().unwrap();
		let mut t =
			block_on(conn_lock.begin()).map_err(|e| MetastoreError::DbError(Box::new(e)))?;

		// Find table and column name to modify
		let (table_name, column_name, is_not_null): (String, String, bool) = {
			let res = block_on(
				sqlx::query(
					"
				SELECT meta_classes.id, meta_attributes.id, is_not_null
				FROM meta_attributes
				INNER JOIN meta_classes ON meta_classes.id = meta_attributes.class_id
				WHERE meta_attributes.id=?;
				",
				)
				.bind(u32::from(attr))
				.fetch_one(&mut *t),
			);

			match res {
				Err(sqlx::Error::RowNotFound) => Err(MetastoreError::BadAttrHandle),
				Err(e) => Err(MetastoreError::DbError(Box::new(e))),
				Ok(res) => {
					let class_id: u32 = res.get("meta_classes.id");
					let attr_id: u32 = res.get("meta_attributes.id");

					Ok((
						Self::get_table_name(class_id.into()),
						Self::get_column_name(attr_id.into()),
						res.get::<bool, _>("is_not_null"),
					))
				}
			}
		}?;

		// Check "not none" constraint
		// Unique constraint is checked later.
		if is_not_null && data.is_none() {
			return Err(MetastoreError::NotNoneViolated);
		}

		// Update data
		{
			let q_str = match data {
				MetastoreData::None(_) => {
					format!("UPDATE \"{table_name}\" SET \"{column_name}\" = NULL;")
				}
				_ => format!("UPDATE \"{table_name}\" SET \"{column_name}\" = ?;"),
			};
			let q = sqlx::query(&q_str);
			let q = Self::bind_storage(q, &mut data);

			// Handle errors
			match block_on(q.execute(&mut *t)) {
				Err(sqlx::Error::Database(e)) => {
					if e.is_unique_violation() {
						return Err(MetastoreError::UniqueViolated);
					} else {
						return Err(MetastoreError::DbError(Box::new(sqlx::Error::Database(e))));
					}
				}
				Err(e) => return Err(MetastoreError::DbError(Box::new(e))),
				Ok(_) => {}
			};
		};

		// Commit transaction
		block_on(t.commit()).map_err(|e| MetastoreError::DbError(Box::new(e)))?;

		Ok(())
	}

	fn class_set_name(&self, _class: ClassHandle, _name: &str) -> Result<(), MetastoreError> {
		unimplemented!()
	}

	fn class_get_name(&self, _class: ClassHandle) -> Result<&str, MetastoreError> {
		unimplemented!()
	}

	fn class_num_attrs(&self, _class: ClassHandle) -> Result<usize, MetastoreError> {
		unimplemented!()
	}

	fn class_get_attrs(
		&self,
		class: ClassHandle,
	) -> Result<Vec<(AttrHandle, SmartString<LazyCompact>, MetastoreDataStub)>, MetastoreError> {
		let mut conn = self.conn.lock().unwrap();

		let res = block_on(
			sqlx::query(
				"
			SELECT id, pretty_name, data_type
			FROM meta_attributes WHERE class_id=?
			ORDER BY id;
			",
			)
			.bind(u32::from(class))
			.fetch_all(&mut *conn),
		);

		let res = match res {
			Err(sqlx::Error::RowNotFound) => return Err(MetastoreError::BadClassHandle),
			Err(e) => return Err(MetastoreError::DbError(Box::new(e))),
			Ok(res) => res,
		};

		Ok(res
			.into_iter()
			.map(|r| {
				let id: u32 = r.get("id");
				let name: &str = r.get("pretty_name");
				let data_type: &str = r.get("data_type");

				(
					id.into(),
					name.into(),
					serde_json::from_str(data_type).unwrap(),
				)
			})
			.collect())
	}

	fn class_get_backlinks(
		&self,
		class: ClassHandle,
	) -> Result<Vec<(ClassHandle, SmartString<LazyCompact>)>, MetastoreError> {
		let classes = self.get_all_classes()?;
		let mut out = Vec::new();
		for (c_handle, c_name) in classes {
			for (_, _, a_type) in self.class_get_attrs(c_handle)? {
				match a_type {
					MetastoreDataStub::Reference { class: ref_class } => {
						if class == ref_class {
							out.push((c_handle, c_name.clone()));
							// We must include each class exactly once
							break;
						}
					}
					_ => {}
				}
			}
		}

		return Ok(out);
	}

	fn attr_set_name(&self, _attr: AttrHandle, _name: &str) -> Result<(), MetastoreError> {
		unimplemented!()
	}

	fn attr_get_name(&self, _attr: AttrHandle) -> Result<&str, MetastoreError> {
		unimplemented!()
	}

	fn attr_get_type(&self, attr: AttrHandle) -> Result<MetastoreDataStub, MetastoreError> {
		let mut conn = self.conn.lock().unwrap();

		let res = block_on(
			sqlx::query("SELECT data_type FROM meta_attributes WHERE id=?;")
				.bind(u32::from(attr))
				.fetch_one(&mut *conn),
		);

		return match res {
			Err(e) => Err(MetastoreError::DbError(Box::new(e))),
			Ok(res) => {
				let type_string = res.get::<String, _>("data_type");
				Ok(serde_json::from_str(&type_string).unwrap())
			}
		};
	}

	fn attr_get_class(&self, _attr: AttrHandle) -> ClassHandle {
		unimplemented!()
	}

	fn find_item_with_attr(
		&self,
		attr: AttrHandle,
		mut attr_value: MetastoreData,
	) -> Result<Option<ItemHandle>, MetastoreError> {
		let mut conn = self.conn.lock().unwrap();

		// Find table and column name to modify
		let (table_name, column_name): (String, String) = {
			let res = block_on(
				sqlx::query(
					"
					SELECT meta_classes.id AS class_id,
					meta_attributes.id AS attr_id
					FROM meta_attributes
					INNER JOIN meta_classes ON meta_classes.id = meta_attributes.class_id
					WHERE meta_attributes.id=?;
					",
				)
				.bind(u32::from(attr))
				.fetch_one(&mut *conn),
			);

			match res {
				Err(sqlx::Error::RowNotFound) => Err(MetastoreError::BadAttrHandle),
				Err(e) => Err(MetastoreError::DbError(Box::new(e))),
				Ok(res) => {
					let class_id: u32 = res.get("class_id");
					let attr_id: u32 = res.get("attr_id");

					Ok((
						Self::get_table_name(class_id.into()),
						Self::get_column_name(attr_id.into()),
					))
				}
			}
		}?;

		let query_str = format!("SELECT id FROM \"{table_name}\" WHERE \"{column_name}\"=?;");
		let mut q = sqlx::query(&query_str);
		q = Self::bind_storage(q, &mut attr_value);

		let res = block_on(q.bind(u32::from(attr)).fetch_one(&mut *conn));
		return match res {
			Err(sqlx::Error::RowNotFound) => Ok(None),
			Err(e) => Err(MetastoreError::DbError(Box::new(e))),
			Ok(res) => {
				let id = res.get::<u32, _>("id");
				Ok(Some(id.into()))
			}
		};
	}
}
