use itertools::Itertools;
use sqlx::{
	query::Query,
	sqlite::{SqliteArguments, SqliteRow},
	Connection, Row, Sqlite,
};
use std::{io::Read, iter, str::FromStr, sync::Arc};
use tracing::debug;
use ufo_ds_core::{
	api::{
		blob::{BlobHandle, Blobstore},
		meta::{AttrInfo, AttributeOptions, ClassInfo, ItemData, Metastore},
	},
	data::{MetastoreData, MetastoreDataStub},
	errors::MetastoreError,
	handles::{AttrHandle, ClassHandle, ItemIdx},
};
use ufo_util::mime::MimeType;

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
			MetastoreData::Integer(x) => q.bind(&*x),
			MetastoreData::PositiveInteger(x) => q.bind(i64::from_be_bytes(x.to_be_bytes())),
			MetastoreData::Boolean(x) => q.bind(*x),
			MetastoreData::Float(x) => q.bind(&*x),
			MetastoreData::Hash { data, .. } => q.bind(&**data),
			MetastoreData::Binary { data, format } => {
				let s = format.to_string();
				let l = u32::try_from(s.len()).unwrap();

				let mut d = Vec::new();
				// Save as [type length][type bytes][data...]
				l.to_be_bytes()
					.chain(s.as_bytes())
					.chain(&data[..])
					.read_to_end(&mut d)
					.unwrap();
				q.bind(d)
			}
			MetastoreData::Reference { item, .. } => q.bind(u32::from(*item)),
			MetastoreData::Blob { handle } => q.bind(u32::from(*handle)),
		}
	}

	fn read_storage(row: &SqliteRow, attr: &AttrInfo) -> MetastoreData {
		let col_name = Self::get_column_name(attr.handle);
		return match attr.data_type {
			MetastoreDataStub::Float => MetastoreData::Float(row.get(&col_name[..])),
			MetastoreDataStub::Boolean => MetastoreData::Boolean(row.get(&col_name[..])),
			MetastoreDataStub::Integer => MetastoreData::Integer(row.get(&col_name[..])),
			MetastoreDataStub::PositiveInteger => MetastoreData::PositiveInteger(
				u64::from_be_bytes(row.get::<i64, _>(&col_name[..]).to_be_bytes()),
			),
			MetastoreDataStub::Text => {
				MetastoreData::Text(Arc::new(row.get::<String, _>(&col_name[..])))
			}
			MetastoreDataStub::Reference { class } => MetastoreData::Reference {
				class,
				item: row.get::<u32, _>(&col_name[..]).into(),
			},
			MetastoreDataStub::Hash { hash_type } => MetastoreData::Hash {
				format: hash_type,
				data: Arc::new(row.get(&col_name[..])),
			},
			MetastoreDataStub::Blob => MetastoreData::Blob {
				handle: row.get::<u32, _>(&col_name[..]).into(),
			},
			MetastoreDataStub::Binary => {
				// TODO: don't panic on malformed db
				let data: Vec<u8> = row.get(&col_name[..]);
				let len = u32::from_be_bytes(data[0..4].try_into().unwrap());
				let len = usize::try_from(len).unwrap();
				let mime = String::from_utf8(data[4..len + 4].into()).unwrap();
				let data = Arc::new(data[len + 4..].into());

				MetastoreData::Binary {
					format: MimeType::from_str(&mime).unwrap(),
					data,
				}
			}
		};
	}

	/// Delete all blobs that are not referenced by an attribute
	async fn delete_dead_blobs(&self) -> Result<(), MetastoreError> {
		let attrs = self.get_all_attrs().await?;
		let mut all_blobs = self
			.all_blobs()
			.map_err(|e| MetastoreError::BlobstoreError(e))?;

		// Do this after getting attrs to prevent deadlock
		let mut conn = self.conn.lock().await;

		// Get all used blobs
		for attr in attrs {
			match attr.data_type {
				MetastoreDataStub::Blob => {
					let table_name = Self::get_table_name(attr.class);
					let column_name = Self::get_column_name(attr.handle);

					let res = sqlx::query(&format!(
						"SELECT \"{column_name}\" FROM \"{table_name}\" ORDER BY id;"
					))
					.fetch_all(&mut *conn)
					.await
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
	async fn add_attr(
		&self,
		class: ClassHandle,
		attr_name: &str,
		data_type: MetastoreDataStub,
		options: AttributeOptions,
	) -> Result<AttrHandle, MetastoreError> {
		// Start transaction
		let mut conn_lock = self.conn.lock().await;
		let mut t = (conn_lock.begin().await).map_err(|e| MetastoreError::DbError(Box::new(e)))?;

		// Add attribute metadata
		let new_attr = {
			let res = sqlx::query(
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
				format!(" REFERENCES \"{}\"(id)", Self::get_table_name(class))
			}

			MetastoreDataStub::Blob => {
				format!(" REFERENCES meta_blobs(id)")
			}
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

		Ok(u32::try_from(new_attr).unwrap().into())
	}

	async fn add_class(&self, class_name: &str) -> Result<ClassHandle, MetastoreError> {
		// Start transaction
		let mut conn_lock = self.conn.lock().await;
		let mut t = conn_lock
			.begin()
			.await
			.map_err(|e| MetastoreError::DbError(Box::new(e)))?;

		// Add metadata
		let new_class_id = {
			let res = sqlx::query("INSERT INTO meta_classes (pretty_name) VALUES (?);")
				.bind(class_name)
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

		return Ok(u32::try_from(new_class_id).unwrap().into());
	}

	async fn add_item(
		&self,
		class: ClassHandle,
		mut attrs: Vec<(AttrHandle, MetastoreData)>,
	) -> Result<ItemIdx, MetastoreError> {
		// Start transaction
		let mut conn_lock = self.conn.lock().await;
		let mut t = conn_lock
			.begin()
			.await
			.map_err(|e| MetastoreError::DbError(Box::new(e)))?;

		let table_name = Self::get_table_name(class);

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
				q = Self::bind_storage(q, value);
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
		let mut conn_lock = self.conn.lock().await;
		let mut t = conn_lock
			.begin()
			.await
			.map_err(|e| MetastoreError::DbError(Box::new(e)))?;

		// Get this attributes' class
		let class_id: ClassHandle = {
			let res = sqlx::query("SELECT class_id FROM meta_attributes WHERE id=?;")
				.bind(u32::from(attr))
				.fetch_one(&mut *t)
				.await;

			match res {
				Err(e) => return Err(MetastoreError::DbError(Box::new(e))),
				Ok(res) => res.get::<u32, _>("class_id").into(),
			}
		};

		// Get the table we want to modify
		let table_name = Self::get_table_name(class_id);
		let col_name = Self::get_column_name(attr);

		// Delete attribute metadata
		if let Err(e) = sqlx::query("DELETE FROM meta_attributes WHERE id=?;")
			.bind(u32::from(attr))
			.execute(&mut *t)
			.await
		{
			return Err(MetastoreError::DbError(Box::new(e)));
		};

		// Delete attribute column
		let q_str = format!("ALTER TABLE \"{table_name}\" DROP COLUMN \"{col_name}\";");
		if let Err(e) = sqlx::query(&q_str).execute(&mut *t).await {
			return Err(MetastoreError::DbError(Box::new(e)));
		};

		// Finish
		if let Err(e) = t.commit().await {
			return Err(MetastoreError::DbError(Box::new(e)));
		};

		// Clean up dangling blobs
		// This locks our connection, so we must drop our existing lock first.
		drop(conn_lock);
		self.delete_dead_blobs().await?;

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

		// Start transaction
		let mut conn_lock = self.conn.lock().await;
		let mut t = conn_lock
			.begin()
			.await
			.map_err(|e| MetastoreError::DbError(Box::new(e)))?;

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

		// Clean up dangling blobs
		// This locks our connection, so we must drop our existing lock first.
		drop(conn_lock);
		self.delete_dead_blobs().await?;

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
		let mut conn = self.conn.lock().await;

		let res = sqlx::query("SELECT id, class_id, pretty_name, data_type FROM meta_attributes WHERE class_id=? AND pretty_name=?;")
			.bind(u32::from(class))
			.bind(attr_name)
			.fetch_one(&mut *conn)
			.await;

		return match res {
			Err(sqlx::Error::RowNotFound) => Ok(None),
			Err(e) => Err(MetastoreError::DbError(Box::new(e))),
			Ok(res) => Ok(Some(AttrInfo {
				handle: res.get::<u32, _>("id").into(),
				class: res.get::<u32, _>("class_id").into(),
				name: res.get::<String, _>("pretty_name").into(),
				data_type: serde_json::from_str(res.get::<&str, _>("data_type")).unwrap(),
			})),
		};
	}

	async fn get_attr(&self, attr: AttrHandle) -> Result<AttrInfo, MetastoreError> {
		let mut conn = self.conn.lock().await;
		let res = sqlx::query(
			"SELECT id, class_id, pretty_name, data_type FROM meta_attributes WHERE id=?;",
		)
		.bind(u32::from(attr))
		.fetch_one(&mut *conn)
		.await;

		return match res {
			Err(sqlx::Error::RowNotFound) => Err(MetastoreError::BadAttrHandle),
			Err(e) => Err(MetastoreError::DbError(Box::new(e))),
			Ok(res) => Ok(AttrInfo {
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
		let mut conn = self.conn.lock().await;

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
		let mut conn = self.conn.lock().await;

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
		let mut conn = self.conn.lock().await;

		let res = sqlx::query(
			"SELECT id, class_id, pretty_name, data_type FROM meta_attributes ORDER BY id;",
		)
		.fetch_all(&mut *conn)
		.await
		.map_err(|e| MetastoreError::DbError(Box::new(e)))?;

		return Ok(res
			.into_iter()
			.map(|res| AttrInfo {
				handle: res.get::<u32, _>("id").into(),
				class: res.get::<u32, _>("class_id").into(),
				name: res.get::<String, _>("pretty_name").into(),
				data_type: serde_json::from_str(res.get::<&str, _>("data_type")).unwrap(),
			})
			.collect());
	}

	async fn get_all_classes(&self) -> Result<Vec<ClassInfo>, MetastoreError> {
		let mut conn = self.conn.lock().await;

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

	/*
		async fn item_set_attr(
			&self,
			attr: AttrHandle,
			mut data: MetastoreData,
		) -> Result<(), MetastoreError> {
			// Start transaction
			let mut conn_lock = self.conn.lock().await;
			let mut t = conn_lock
				.begin()
				.await
				.map_err(|e| MetastoreError::DbError(Box::new(e)))?;

			// Find table and column name to modify
			let (table_name, column_name, is_not_null): (String, String, bool) = {
				let res = sqlx::query(
					"
					SELECT meta_classes.id, meta_attributes.id, is_not_null
					FROM meta_attributes
					INNER JOIN meta_classes ON meta_classes.id = meta_attributes.class_id
					WHERE meta_attributes.id=?;
					",
				)
				.bind(u32::from(attr))
				.fetch_one(&mut *t)
				.await;

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
				match q.execute(&mut *t).await {
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
			t.commit()
				.await
				.map_err(|e| MetastoreError::DbError(Box::new(e)))?;

			Ok(())
		}
	*/

	async fn class_set_name(&self, _class: ClassHandle, _name: &str) -> Result<(), MetastoreError> {
		unimplemented!()
	}

	async fn class_num_attrs(&self, _class: ClassHandle) -> Result<usize, MetastoreError> {
		unimplemented!()
	}

	async fn class_get_attrs(&self, class: ClassHandle) -> Result<Vec<AttrInfo>, MetastoreError> {
		let res = sqlx::query(
			"
			SELECT id, pretty_name, data_type, class_id
			FROM meta_attributes WHERE class_id=?
			ORDER BY id;
			",
		)
		.bind(u32::from(class))
		.fetch_all(&mut *self.conn.lock().await)
		.await;

		let res = match res {
			Err(sqlx::Error::RowNotFound) => return Err(MetastoreError::BadClassHandle),
			Err(e) => return Err(MetastoreError::DbError(Box::new(e))),
			Ok(res) => res,
		};

		Ok(res
			.into_iter()
			.map(|x| AttrInfo {
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
				match attr.data_type {
					MetastoreDataStub::Reference { class: ref_class } => {
						if class == ref_class {
							out.push(ClassInfo {
								handle: i_class.handle,
								name: i_class.name,
							});
							// We include each class exactly once, so break here.
							break;
						}
					}
					_ => {}
				}
			}
		}

		return Ok(out);
	}

	async fn attr_set_name(&self, _attr: AttrHandle, _name: &str) -> Result<(), MetastoreError> {
		unimplemented!()
	}

	async fn find_item_with_attr(
		&self,
		attr: AttrHandle,
		mut attr_value: MetastoreData,
	) -> Result<Option<ItemIdx>, MetastoreError> {
		let mut conn = self.conn.lock().await;

		// Find table and column name to modify
		let column_name = Self::get_column_name(attr.into());
		let table_name: String = {
			// TODO: meta_attributes.id AS attr_id
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
		q = Self::bind_storage(q, &mut attr_value);

		let res = q.bind(u32::from(attr)).fetch_one(&mut *conn).await;
		return match res {
			Err(sqlx::Error::RowNotFound) => Ok(None),
			Err(e) => Err(MetastoreError::DbError(Box::new(e))),
			Ok(res) => Ok(Some(res.get::<u32, _>("id").into())),
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

		// Start transaction
		let mut conn_lock = self.conn.lock().await;

		let table_name = Self::get_table_name(class);

		let res = sqlx::query(&format!(
				"SELECT * FROM \"{table_name}\" ORDER BY id LIMIT \"{page_size}\" OFFSET \"{start_at}\" ;"
			))
		.fetch_all(&mut *conn_lock)
		.await
		.map_err(|e| MetastoreError::DbError(Box::new(e)))?;

		let mut out = Vec::new();
		for row in res {
			out.push(ItemData {
				handle: row.get::<u32, _>("id").into(),
				attrs: attrs
					.iter()
					.map(|attr| Self::read_storage(&row, attr))
					.collect(),
			})
		}

		return Ok(out);
	}
}
