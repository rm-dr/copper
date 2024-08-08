use crate::{
	api::{AttrHandle, AttributeOptions, ClassHandle, Dataset, ItemHandle},
	data::{StorageData, StorageDataType},
	errors::DatasetError,
};
use base64::{
	alphabet::Alphabet,
	engine::{DecodePaddingMode, GeneralPurpose, GeneralPurposeConfig},
	Engine,
};
use futures::executor::block_on;
use itertools::Itertools;
use smartstring::{LazyCompact, SmartString};
use sqlx::{
	query::Query, sqlite::SqliteArguments, Connection, Executor, Row, Sqlite, SqliteConnection,
};
use std::iter;

pub struct SQLiteDataset {
	/// The path to the database we'll connect to
	database: SmartString<LazyCompact>,

	/// A connection to a database.
	/// `None` if disconnected.
	conn: Option<SqliteConnection>,

	/// Base64 sanitizer for user-provided strings.
	name_sanitizer: GeneralPurpose,
}

impl SQLiteDataset {
	pub fn new(database: &str) -> Self {
		// Nonstandard base 64
		let name_sanitizer = GeneralPurpose::new(
			&Alphabet::new("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-+")
				.unwrap(),
			GeneralPurposeConfig::new()
				.with_encode_padding(false)
				.with_decode_padding_mode(DecodePaddingMode::RequireNone),
		);

		Self {
			database: database.into(),
			conn: None,
			name_sanitizer,
		}
	}

	pub fn connect(&mut self) -> Result<(), DatasetError> {
		let conn = block_on(SqliteConnection::connect(&self.database))?;
		self.conn = Some(conn);

		block_on(sqlx::query(include_str!("./init_db.sql")).execute(self.conn.as_mut().unwrap()))
			.unwrap();

		// TODO: load & check metadata, don't destroy db
		Ok(())
	}

	/// Characters that won't be sanitized in table and column names
	const ALLOWED_NAME_CHARS: &'static str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

	/// Sanitize a user-provided string for use as an sql table or column name.
	///
	/// This may still contain a keyword like "JOIN". To prevent problems, always
	/// put table and column names in quotes when writing queries.
	#[inline(always)]
	fn sanitize_name(&self, prefix: &str, name: &str) -> String {
		let mut all_ok = true;
		for i in name.chars() {
			if !Self::ALLOWED_NAME_CHARS.contains(i) {
				all_ok = false;
				break;
			}
		}

		if all_ok {
			format!("{prefix}_{name}")
		} else {
			format!("{prefix}_{}", self.name_sanitizer.encode(name.as_bytes()))
		}
	}
}

// SQL helper functions
impl SQLiteDataset {
	fn get_table_name<'e, 'c, E>(executor: E, class: ClassHandle) -> Result<String, DatasetError>
	where
		E: Executor<'c, Database = Sqlite>,
	{
		// Find table to modify
		let table_name: String = {
			let res = block_on(
				sqlx::query("SELECT table_name FROM meta_classes WHERE id=?;")
					.bind(u32::from(class))
					.fetch_one(executor),
			);

			match res {
				Err(sqlx::Error::RowNotFound) => {
					return Err(DatasetError::BadClassHandle);
				}
				Err(e) => return Err(e.into()),
				Ok(res) => res.get("table_name"),
			}
		};

		Ok(table_name)
	}

	fn bind_storage<'a>(
		q: Query<'a, Sqlite, SqliteArguments<'a>>,
		storage: &'a StorageData,
	) -> Query<'a, Sqlite, SqliteArguments<'a>> {
		match storage {
			StorageData::None(_) => q,
			StorageData::Text(s) => q.bind(s),
			StorageData::Path(p) => q.bind(p),
			// TODO: store as int for easy compare
			StorageData::Integer(x) => q.bind(x.to_le_bytes().to_vec()),
			StorageData::PositiveInteger(x) => q.bind(x.to_le_bytes().to_vec()),
			StorageData::Float(x) => q.bind(x),
			StorageData::Hash { data, .. } => q.bind(data.clone()),
			StorageData::Binary { data, .. } => q.bind(data.clone()),
			StorageData::Reference { item, .. } => q.bind(u32::from(*item)),
		}
	}
}

impl Dataset for SQLiteDataset {
	fn add_attr(
		&mut self,
		class: ClassHandle,
		attr_name: &str,
		data_type: StorageDataType,
		options: AttributeOptions,
	) -> Result<AttrHandle, DatasetError> {
		let column_name = self.sanitize_name("attr", attr_name);

		// Start transaction
		let mut t = if let Some(ref mut conn) = self.conn {
			block_on(conn.begin())?
		} else {
			return Err(DatasetError::NotConnected);
		};

		// Add attribute metadata
		let new_attr_id = {
			let res = block_on(
				sqlx::query(
					"
				INSERT INTO meta_attributes (
					class_id, column_name, pretty_name, data_type,
					is_unique, is_not_null
				) VALUES (?, ?, ?, ?, ?, ?);
				",
				)
				.bind(u32::from(class))
				.bind(&column_name)
				.bind(&attr_name)
				.bind(data_type.to_db_str())
				.bind(options.unique)
				.bind(options.not_null)
				.execute(&mut *t),
			);

			match res {
				Err(sqlx::Error::Database(e)) => {
					if e.is_unique_violation() {
						return Err(DatasetError::DuplicateAttrName(attr_name.into()));
					} else {
						return Err(sqlx::Error::Database(e).into());
					}
				}
				Err(e) => return Err(e.into()),
				Ok(x) => x.last_insert_rowid(),
			}
		};

		// Find table to modify
		let table_name = Self::get_table_name(&mut *t, class)?;

		// Map internal type to sqlite type
		let data_type_str = match data_type {
			StorageDataType::Text => "TEXT",
			StorageDataType::Path => "TEXT",
			StorageDataType::Integer => "BLOB",
			StorageDataType::PositiveInteger => "BLOB",
			StorageDataType::Float => "REAL",
			StorageDataType::Binary => "BLOB",
			StorageDataType::Reference { .. } => "INTEGER",
			StorageDataType::Hash { .. } => "BLOB",
		};

		let not_null = if options.not_null { " NOT NULL" } else { "" };

		// Add foreign key if necessary
		let references = match data_type {
			StorageDataType::Reference { class } => {
				let table_name: String = {
					let res = block_on(
						sqlx::query("SELECT table_name FROM meta_classes WHERE id=?;")
							.bind(u32::from(class))
							.fetch_one(&mut *t),
					)?;
					res.get("table_name")
				};
				format!(" REFERENCES \"{table_name}\"(id)")
			}
			_ => "".into(),
		};

		// Add new column
		block_on(
			sqlx::query(&format!(
				"ALTER TABLE \"{table_name}\" ADD \"{column_name}\" {data_type_str}{not_null}{references};",
			))
			.execute(&mut *t),
		)?; // This should never fail

		// Add unique constraint if necessary
		if options.unique {
			// TODO: unique name (what if renamed?)
			block_on(
				sqlx::query(&format!(
					"CREATE UNIQUE INDEX \"unique_{table_name}_{column_name}\" ON \"{table_name}\"(\"{column_name}\");",
				))
				.execute(&mut *t),
			)?; // This should never fail
		}

		// Commit transaction
		block_on(t.commit())?;

		Ok(u32::try_from(new_attr_id).unwrap().into())
	}

	fn add_class(&mut self, class_name: &str) -> Result<ClassHandle, DatasetError> {
		let table_name = self.sanitize_name("class", class_name);

		// Start transaction
		let mut t = if let Some(ref mut conn) = self.conn {
			block_on(conn.begin())?
		} else {
			return Err(DatasetError::NotConnected);
		};

		// Add metadata
		let new_class_id = {
			let res = block_on(
				sqlx::query("INSERT INTO meta_classes (table_name, pretty_name) VALUES (?, ?);")
					.bind(&table_name)
					.bind(class_name)
					.execute(&mut *t),
			);

			match res {
				Err(sqlx::Error::Database(e)) => {
					if e.is_unique_violation() {
						return Err(DatasetError::DuplicateClassName(class_name.into()));
					} else {
						return Err(sqlx::Error::Database(e).into());
					}
				}
				Err(e) => return Err(e.into()),
				Ok(res) => res.last_insert_rowid(),
			}
		};

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
		attrs: &[(AttrHandle, StorageData)],
	) -> Result<ItemHandle, DatasetError> {
		// Start transaction
		let mut t = if let Some(ref mut conn) = self.conn {
			block_on(conn.begin())?
		} else {
			return Err(DatasetError::NotConnected);
		};

		let table_name = Self::get_table_name(&mut *t, class)?;

		// Add new row with data
		let res = if attrs.len() == 0 {
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
						sqlx::query("SELECT column_name FROM meta_attributes WHERE id=?;")
							.bind(u32::from(*a))
							.fetch_one(&mut *t),
					);

					let column_name: String = match res {
						Err(sqlx::Error::RowNotFound) => {
							return Err(DatasetError::BadClassHandle);
						}
						Err(e) => return Err(e.into()),
						Ok(res) => res.get("column_name"),
					};

					attr_names.push(format!("\"{column_name}\""));
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
					return Err(DatasetError::UniqueViolated);
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

	fn del_attr(&mut self, _attr: AttrHandle) -> Result<(), DatasetError> {
		unimplemented!()
	}

	fn del_class(&mut self, _class: ClassHandle) -> Result<(), DatasetError> {
		unimplemented!()
	}

	fn del_item(&mut self, _item: ItemHandle) -> Result<(), DatasetError> {
		unimplemented!()
	}

	fn get_attr(
		&mut self,
		class: ClassHandle,
		attr_name: &str,
	) -> Result<Option<AttrHandle>, DatasetError> {
		// Start transaction
		let conn = if let Some(ref mut conn) = self.conn {
			conn
		} else {
			return Err(DatasetError::NotConnected);
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

	fn get_class(&mut self, class_name: &str) -> Result<Option<ClassHandle>, DatasetError> {
		// Start transaction
		let conn = if let Some(ref mut conn) = self.conn {
			conn
		} else {
			return Err(DatasetError::NotConnected);
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
	) -> Result<StorageData, DatasetError> {
		unimplemented!()
	}

	fn item_get_class(&self, _item: ItemHandle) -> Result<ClassHandle, DatasetError> {
		unimplemented!()
	}

	fn item_set_attr(&mut self, attr: AttrHandle, data: &StorageData) -> Result<(), DatasetError> {
		// Start transaction
		let mut t = if let Some(ref mut conn) = self.conn {
			block_on(conn.begin())?
		} else {
			return Err(DatasetError::NotConnected);
		};

		// Find table and column name to modify
		let (table_name, column_name, is_not_null): (String, String, bool) = {
			let res = block_on(
				sqlx::query(
					"
				SELECT column_name, meta_classes.table_name, is_not_null
				FROM meta_attributes
				INNER JOIN meta_classes ON meta_classes.id = meta_attributes.class_id
				WHERE meta_attributes.id=?;
				",
				)
				.bind(u32::from(attr))
				.fetch_one(&mut *t),
			);

			match res {
				Err(sqlx::Error::RowNotFound) => Err(DatasetError::BadAttrHandle),
				Err(e) => Err(e.into()),
				Ok(res) => Ok((
					res.get("table_name"),
					res.get("column_name"),
					res.get::<bool, _>("is_not_null"),
				)),
			}
		}?;

		// Check "not none" constraint
		// Unique constraint is checked later.
		if is_not_null && data.is_none() {
			return Err(DatasetError::NotNoneViolated);
		}

		// Update data
		{
			let q_str = match data {
				StorageData::None(_) => {
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
						return Err(DatasetError::UniqueViolated);
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

	fn class_set_name(&mut self, _class: ClassHandle, _name: &str) -> Result<(), DatasetError> {
		unimplemented!()
	}

	fn class_get_name(&self, _class: ClassHandle) -> Result<&str, DatasetError> {
		unimplemented!()
	}

	fn class_num_attrs(&self, _class: ClassHandle) -> Result<usize, DatasetError> {
		unimplemented!()
	}

	fn attr_set_name(&mut self, _attr: AttrHandle, _name: &str) -> Result<(), DatasetError> {
		unimplemented!()
	}

	fn attr_get_name(&self, _attr: AttrHandle) -> Result<&str, DatasetError> {
		unimplemented!()
	}

	fn attr_get_type(&self, _attr: AttrHandle) -> Result<StorageDataType, DatasetError> {
		todo!()
	}

	fn attr_get_class(&self, _attr: AttrHandle) -> ClassHandle {
		unimplemented!()
	}
}
