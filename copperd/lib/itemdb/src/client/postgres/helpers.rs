use itertools::Itertools;
use sqlx::Row;

use crate::{
	client::base::errors::transaction::ApplyTransactionError, transaction::AddItemError, AttrData,
	AttrDataStub, AttributeId, AttributeOptions, ClassId, ItemId,
};

pub(super) enum SqlxOrItemError {
	Sqlx(sqlx::Error),
	AddItem(AddItemError),
}

impl From<sqlx::Error> for SqlxOrItemError {
	fn from(value: sqlx::Error) -> Self {
		Self::Sqlx(value)
	}
}

impl From<AddItemError> for SqlxOrItemError {
	fn from(value: AddItemError) -> Self {
		Self::AddItem(value)
	}
}

impl From<SqlxOrItemError> for ApplyTransactionError {
	fn from(value: SqlxOrItemError) -> Self {
		match value {
			SqlxOrItemError::AddItem(x) => x.into(),
			SqlxOrItemError::Sqlx(x) => x.into(),
		}
	}
}

pub(super) async fn add_item(
	t: &mut sqlx::Transaction<'_, sqlx::Postgres>,
	to_class: ClassId,
	attributes: Vec<(AttributeId, AttrData)>,
) -> Result<ItemId, SqlxOrItemError> {
	// Make sure we have at most one of each attribute
	if !attributes.iter().map(|(x, _)| x).all_unique() {
		return Err(AddItemError::RepeatedAttribute.into());
	}

	let res = sqlx::query("INSERT INTO item (class_id) VALUES ($1) RETURNING id;")
		.bind(i64::from(to_class))
		.fetch_one(&mut **t)
		.await;

	// Create the new item, we attach attributes afterwards
	let new_item: ItemId = match res {
		Ok(res) => res.get::<i64, _>("id").into(),
		Err(sqlx::Error::Database(e)) => {
			if e.is_foreign_key_violation() {
				return Err(AddItemError::NoSuchClass.into());
			} else {
				return Err(sqlx::Error::Database(e).into());
			}
		}
		Err(e) => return Err(e.into()),
	};

	// Create instances for every attribute that was provided
	for (attr, value) in attributes {
		// Make sure this attribute comes from this class,
		// and get its details
		let (data_type, attr_options) = match sqlx::query(
			"SELECT data_type, is_unique, is_not_null FROM attribute WHERE id=$1 AND class_id=$2;",
		)
		.bind(i64::from(attr))
		.bind(i64::from(to_class))
		.fetch_one(&mut **t)
		.await
		{
			Ok(res) => {
				let data_type: AttrDataStub =
					serde_json::from_str(&res.get::<String, _>("data_type")).unwrap();

				(
					data_type,
					AttributeOptions {
						is_not_null: res.get("is_not_null"),
						is_unique: res.get("is_unique"),
					},
				)
			}

			Err(sqlx::Error::RowNotFound) => return Err(AddItemError::ForeignAttribute.into()),
			Err(e) => return Err(e.into()),
		};

		// Make sure type matches
		if data_type != value.as_stub() {
			return Err(AddItemError::AttributeDataTypeMismatch.into());
		}

		// Check "not null" constraint
		if attr_options.is_not_null && value.is_none() {
			return Err(AddItemError::NotNullViolated.into());
		}

		let value_ser = serde_json::to_string(&value).unwrap();

		// Check "unique" constraint
		// Has no effect on blobs, so don't check them.
		// (this is why that switch is hidden in ui)
		if attr_options.is_unique && value.as_stub() != AttrDataStub::Blob {
			// Look for non-unique row
			match sqlx::query(
				"
				SELECT COUNT(*) FROM attribute_instance
				WHERE attribute_id=$1
				AND attribute_value=$2
				",
			)
			.bind(i64::from(attr))
			.bind(&value_ser)
			.fetch_one(&mut **t)
			.await
			{
				Ok(res) => {
					let count: i64 = res.get("count");
					if count != 0 {
						return Err(AddItemError::UniqueViolated.into());
					}
				}

				Err(e) => return Err(e.into()),
			};
		}

		// Create the attribute instance
		let res = sqlx::query(
			"INSERT INTO attribute_instance (item_id, attribute_id, attribute_value)
			VALUES ($1, $2, $3);",
		)
		.bind(i64::from(new_item))
		.bind(i64::from(attr))
		.bind(&value_ser)
		.execute(&mut **t)
		.await;

		match res {
			Ok(_) => {}
			Err(sqlx::Error::Database(e)) => {
				if e.is_foreign_key_violation() {
					return Err(AddItemError::BadAttribute.into());
				} else {
					return Err(sqlx::Error::Database(e).into());
				}
			}
			Err(e) => return Err(e.into()),
		};
	}

	return Ok(new_item);
}
