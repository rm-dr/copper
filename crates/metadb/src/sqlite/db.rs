use crate::{
	api::{AttrHandle, AttributeOptions, ClassHandle, ItemHandle, MetaDb},
	data::{MetaDbData, MetaDbDataStub},
	errors::MetaDbError,
};
use futures::executor::block_on;
use itertools::Itertools;
use smartstring::{LazyCompact, SmartString};
use sqlx::{
	query::Query, sqlite::SqliteArguments, Connection, Executor, Row, Sqlite, SqliteConnection,
};
use std::iter;

pub struct SQLiteMetaDB {
	/// The path to the database we'll connect to
	database: SmartString<LazyCompact>,

	/// A connection to a database.
	/// `None` if disconnected.
	conn: Option<SqliteConnection>,
}

impl SQLiteMetaDB {
	pub fn new(database: &str) -> Self {
		Self {
			database: database.into(),
			conn: None,
		}
	}

	pub fn connect(&mut self) -> Result<(), MetaDbError> {
		let mut conn = block_on(SqliteConnection::connect(&self.database))?;

		block_on(sqlx::query(include_str!("./init_db.sql")).execute(&mut conn)).unwrap();

		block_on(
			sqlx::query("INSERT INTO meta_meta (var, val) VALUES (?, ?);")
				.bind("ufo_version")
				.bind(env!("CARGO_PKG_VERSION"))
				.execute(&mut conn),
		)
		.unwrap();

		self.conn = Some(conn);

		// TODO: load & check metadata, don't destroy db
		Ok(())
	}
}

// SQL helper functions
impl SQLiteMetaDB {
	fn get_table_name<'e, 'c, E>(executor: E, class: ClassHandle) -> Result<String, MetaDbError>
	where
		E: Executor<'c, Database = Sqlite>,
	{
		let id: u32 = {
			let res = block_on(
				sqlx::query("SELECT id FROM meta_classes WHERE id=?;")
					.bind(u32::from(class))
					.fetch_one(executor),
			);

			match res {
				Err(sqlx::Error::RowNotFound) => {
					return Err(MetaDbError::BadClassHandle);
				}
				Err(e) => return Err(e.into()),
				Ok(res) => res.get("id"),
			}
		};

		Ok(format!("class_{id}"))
	}

	fn bind_storage<'a>(
		q: Query<'a, Sqlite, SqliteArguments<'a>>,
		storage: &'a MetaDbData,
	) -> Query<'a, Sqlite, SqliteArguments<'a>> {
		match storage {
			MetaDbData::None(_) => q,
			MetaDbData::Text(s) => q.bind(&**s),
			MetaDbData::Path(p) => q.bind(p.to_str().unwrap()),
			MetaDbData::Integer(x) => q.bind(x),
			MetaDbData::PositiveInteger(x) => q.bind(i64::from_be_bytes(x.to_be_bytes())),
			MetaDbData::Float(x) => q.bind(x),
			MetaDbData::Hash { data, .. } => q.bind((**data).clone()),
			MetaDbData::Binary { data, .. } => q.bind((**data).clone()),
			MetaDbData::Reference { item, .. } => q.bind(u32::from(*item)),
		}
	}
}

impl MetaDb for SQLiteMetaDB {
	fn add_attr(
		&mut self,
		class: ClassHandle,
		attr_name: &str,
		data_type: MetaDbDataStub,
		options: AttributeOptions,
	) -> Result<AttrHandle, MetaDbError> {
		// Start transaction
		let mut t = if let Some(ref mut conn) = self.conn {
			block_on(conn.begin())?
		} else {
			return Err(MetaDbError::NotConnected);
		};

		// Add attribute metadata
		let new_attr_id = {
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
				.bind(data_type.to_db_str())
				.bind(options.unique)
				.bind(options.not_null)
				.execute(&mut *t),
			);

			match res {
				Err(sqlx::Error::Database(e)) => {
					if e.is_unique_violation() {
						return Err(MetaDbError::DuplicateAttrName(attr_name.into()));
					} else {
						return Err(sqlx::Error::Database(e).into());
					}
				}
				Err(e) => return Err(e.into()),
				Ok(x) => x.last_insert_rowid(),
			}
		};
		let column_name = format!("attr_{new_attr_id}");

		// Find table to modify
		let table_name = Self::get_table_name(&mut *t, class)?;

		// Map internal type to sqlite type
		let data_type_str = match data_type {
			MetaDbDataStub::Text => "TEXT",
			MetaDbDataStub::Path => "TEXT",
			MetaDbDataStub::Integer => "INTEGER",
			MetaDbDataStub::PositiveInteger => "INTEGER",
			MetaDbDataStub::Float => "REAL",
			MetaDbDataStub::Binary => "BLOB",
			MetaDbDataStub::Reference { .. } => "INTEGER",
			MetaDbDataStub::Hash { .. } => "BLOB",
		};

		let not_null = if options.not_null { " NOT NULL" } else { "" };

		// Add foreign key if necessary
		let references = match data_type {
			MetaDbDataStub::Reference { class } => {
				let id: u32 = {
					let res = block_on(
						sqlx::query("SELECT id FROM meta_classes WHERE id=?;")
							.bind(u32::from(class))
							.fetch_one(&mut *t),
					)?;
					res.get("id")
				};
				format!(" REFERENCES \"class_{id}\"(id)")
			}
			_ => "".into(),
		};

		// Add new column
		block_on(
			sqlx::query(&format!(
				"ALTER TABLE \"{table_name}\" ADD \"{column_name}\" {data_type_str}{not_null}{references};",
			))
			.execute(&mut *t),
		)?;

		// Add unique constraint if necessary
		if options.unique {
			block_on(
				sqlx::query(&format!(
					"CREATE UNIQUE INDEX \"unique_{table_name}_{column_name}\" ON \"{table_name}\"(\"{column_name}\");",
				))
				.execute(&mut *t),
			)?;
		}

		// Commit transaction
		block_on(t.commit())?;

		Ok(u32::try_from(new_attr_id).unwrap().into())
	}

	fn add_class(&mut self, class_name: &str) -> Result<ClassHandle, MetaDbError> {
		// Start transaction
		let mut t = if let Some(ref mut conn) = self.conn {
			block_on(conn.begin())?
		} else {
			return Err(MetaDbError::NotConnected);
		};

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
						return Err(MetaDbError::DuplicateClassName(class_name.into()));
					} else {
						return Err(sqlx::Error::Database(e).into());
					}
				}
				Err(e) => return Err(e.into()),
				Ok(res) => res.last_insert_rowid(),
			}
		};
		let table_name = format!("class_{new_class_id}");

		// Create new table
		block_on(
			sqlx::query(&format!(
				"CREATE TABLE IF NOT EXISTS \"{table_name}\" (id INTEGER PRIMARY KEY NOT NULL);"
			))
			.execute(&mut *t),
		)?;

		// Commit transaction
		block_on(t.commit())?;

		return Ok(u32::try_from(new_class_id).unwrap().into());
	}

	fn add_item(
		&mut self,
		class: ClassHandle,
		attrs: &[(AttrHandle, MetaDbData)],
	) -> Result<ItemHandle, MetaDbError> {
		// Start transaction
		let mut t = if let Some(ref mut conn) = self.conn {
			block_on(conn.begin())?
		} else {
			return Err(MetaDbError::NotConnected);
		};

		let table_name = Self::get_table_name(&mut *t, class)?;

		// Add new row with data
		let res = if attrs.is_empty() {
			// If we were given no attributes
			block_on(
				sqlx::query(&format!("INSERT INTO \"{table_name}\" DEFAULT VALUES;",))
					.execute(&mut *t),
			)
		} else {
			// Find rows of all provided attributes
			let (attr_names, attr_values) = {
				let mut attr_names: Vec<String> = Vec::new();
				for (a, _) in attrs {
					let res = block_on(
						sqlx::query("SELECT id FROM meta_attributes WHERE id=?;")
							.bind(u32::from(*a))
							.fetch_one(&mut *t),
					);

					let column_id: u32 = match res {
						Err(sqlx::Error::RowNotFound) => {
							return Err(MetaDbError::BadClassHandle);
						}
						Err(e) => return Err(e.into()),
						Ok(res) => res.get("id"),
					};

					attr_names.push(format!("\"attr_{column_id}\""));
				}

				(
					attr_names.join(", "),
					iter::repeat('?').take(attr_names.len()).join(", "),
				)
			};

			let q_str =
				format!("INSERT INTO \"{table_name}\" ({attr_names}) VALUES ({attr_values});",);
			let mut q = sqlx::query(&q_str);

			for (_, value) in attrs {
				q = Self::bind_storage(q, value);
			}

			block_on(q.execute(&mut *t))
		};

		// Handle errors
		let id = match res {
			Err(sqlx::Error::Database(e)) => {
				if e.is_unique_violation() {
					return Err(MetaDbError::UniqueViolated);
				} else {
					return Err(sqlx::Error::Database(e).into());
				}
			}
			Err(x) => return Err(x.into()),
			Ok(res) => res.last_insert_rowid(),
		};

		// Commit transaction
		block_on(t.commit())?;

		Ok(u32::try_from(id).unwrap().into())
	}

	fn del_attr(&mut self, _attr: AttrHandle) -> Result<(), MetaDbError> {
		unimplemented!()
	}

	fn del_class(&mut self, _class: ClassHandle) -> Result<(), MetaDbError> {
		unimplemented!()
	}

	fn del_item(&mut self, _item: ItemHandle) -> Result<(), MetaDbError> {
		unimplemented!()
	}

	fn get_attr(
		&mut self,
		class: ClassHandle,
		attr_name: &str,
	) -> Result<Option<AttrHandle>, MetaDbError> {
		// Start transaction
		let conn = if let Some(ref mut conn) = self.conn {
			conn
		} else {
			return Err(MetaDbError::NotConnected);
		};

		let res = block_on(
			sqlx::query("SELECT id FROM meta_attributes WHERE class_id=? AND pretty_name=?;")
				.bind(u32::from(class))
				.bind(attr_name)
				.fetch_one(conn),
		);

		return match res {
			Err(sqlx::Error::RowNotFound) => Ok(None),
			Err(e) => Err(e.into()),
			Ok(res) => Ok(Some(res.get::<u32, _>("id").into())),
		};
	}

	fn get_class(&mut self, class_name: &str) -> Result<Option<ClassHandle>, MetaDbError> {
		// Start transaction
		let conn = if let Some(ref mut conn) = self.conn {
			conn
		} else {
			return Err(MetaDbError::NotConnected);
		};

		let res = block_on(
			sqlx::query("SELECT id FROM meta_classes WHERE pretty_name=?;")
				.bind(class_name)
				.fetch_one(conn),
		);

		return match res {
			Err(sqlx::Error::RowNotFound) => Ok(None),
			Err(e) => Err(e.into()),
			Ok(res) => Ok(Some(res.get::<u32, _>("id").into())),
		};
	}

	fn item_get_attr(
		&self,
		_item: ItemHandle,
		_attr: AttrHandle,
	) -> Result<MetaDbData, MetaDbError> {
		unimplemented!()
	}

	fn item_get_class(&self, _item: ItemHandle) -> Result<ClassHandle, MetaDbError> {
		unimplemented!()
	}

	fn item_set_attr(&mut self, attr: AttrHandle, data: &MetaDbData) -> Result<(), MetaDbError> {
		// Start transaction
		let mut t = if let Some(ref mut conn) = self.conn {
			block_on(conn.begin())?
		} else {
			return Err(MetaDbError::NotConnected);
		};

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
				Err(sqlx::Error::RowNotFound) => Err(MetaDbError::BadAttrHandle),
				Err(e) => Err(e.into()),
				Ok(res) => {
					let class_id: u32 = res.get("meta_classes.id");
					let attr_id: u32 = res.get("meta_attributes.id");

					Ok((
						format!("class_{class_id}"),
						format!("attr_{attr_id}"),
						res.get::<bool, _>("is_not_null"),
					))
				}
			}
		}?;

		// Check "not none" constraint
		// Unique constraint is checked later.
		if is_not_null && data.is_none() {
			return Err(MetaDbError::NotNoneViolated);
		}

		// Update data
		{
			let q_str = match data {
				MetaDbData::None(_) => {
					format!("UPDATE \"{table_name}\" SET \"{column_name}\" = NULL;")
				}
				_ => format!("UPDATE \"{table_name}\" SET \"{column_name}\" = ?;"),
			};
			let q = sqlx::query(&q_str);
			let q = Self::bind_storage(q, data);

			// Handle errors
			match block_on(q.execute(&mut *t)) {
				Err(sqlx::Error::Database(e)) => {
					if e.is_unique_violation() {
						return Err(MetaDbError::UniqueViolated);
					} else {
						return Err(sqlx::Error::Database(e).into());
					}
				}
				Err(x) => return Err(x.into()),
				Ok(_) => {}
			};
		};

		// Commit transaction
		block_on(t.commit())?;

		Ok(())
	}

	fn class_set_name(&mut self, _class: ClassHandle, _name: &str) -> Result<(), MetaDbError> {
		unimplemented!()
	}

	fn class_get_name(&self, _class: ClassHandle) -> Result<&str, MetaDbError> {
		unimplemented!()
	}

	fn class_num_attrs(&self, _class: ClassHandle) -> Result<usize, MetaDbError> {
		unimplemented!()
	}

	fn class_get_attrs(
		&mut self,
		class: ClassHandle,
	) -> Result<Vec<(AttrHandle, SmartString<LazyCompact>, MetaDbDataStub)>, MetaDbError> {
		// Start transaction
		let conn = if let Some(ref mut conn) = self.conn {
			conn
		} else {
			return Err(MetaDbError::NotConnected);
		};

		let res = block_on(
			sqlx::query(
				"
			SELECT id, pretty_name, data_type
			FROM meta_attributes WHERE class_id=?
			ORDER BY id;
			",
			)
			.bind(u32::from(class))
			.fetch_all(conn),
		);

		let res = match res {
			Err(sqlx::Error::RowNotFound) => return Err(MetaDbError::BadClassHandle),
			Err(e) => return Err(e.into()),
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
					MetaDbDataStub::from_db_str(data_type).unwrap(),
				)
			})
			.collect())
	}

	fn attr_set_name(&mut self, _attr: AttrHandle, _name: &str) -> Result<(), MetaDbError> {
		unimplemented!()
	}

	fn attr_get_name(&self, _attr: AttrHandle) -> Result<&str, MetaDbError> {
		unimplemented!()
	}

	fn attr_get_type(&self, _attr: AttrHandle) -> Result<MetaDbDataStub, MetaDbError> {
		todo!()
	}

	fn attr_get_class(&self, _attr: AttrHandle) -> ClassHandle {
		unimplemented!()
	}
}
