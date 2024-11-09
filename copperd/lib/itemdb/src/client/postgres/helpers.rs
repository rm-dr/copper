use itertools::Itertools;
use sqlx::Row;

use crate::{
	client::base::errors::transaction::ApplyTransactionError, transaction::AddItemError, AttrData,
	AttrDataStub, AttributeId, AttributeInfo, AttributeOptions, ClassId, ItemId,
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

	//
	// Create attribute instances
	//

	// Get all attributes this class has
	let all_attrs = sqlx::query("SELECT * FROM attribute WHERE class_id=$1;")
		.bind(i64::from(to_class))
		.fetch_all(&mut **t)
		.await?
		.into_iter()
		.map(|row| AttributeInfo {
			id: row.get::<i64, _>("id").into(),
			class: row.get::<i64, _>("class_id").into(),
			order: row.get::<i64, _>("attr_order"),
			name: row.get::<String, _>("pretty_name").into(),
			data_type: serde_json::from_str(row.get::<&str, _>("data_type")).unwrap(),
			options: AttributeOptions {
				is_unique: row.get("is_unique"),
				is_not_null: row.get("is_not_null"),
			},
		})
		.collect::<Vec<_>>();

	for (attr_id, _) in &attributes {
		// Make sure all attributes we got belong to this class...
		if all_attrs.iter().all(|x| x.id != *attr_id) {
			return Err(SqlxOrItemError::AddItem(AddItemError::ForeignAttribute));
		}

		// ...and that we only saw one instance of each attribute
		if attributes.iter().filter(|(x, _)| x == attr_id).count() != 1 {
			return Err(SqlxOrItemError::AddItem(AddItemError::RepeatedAttribute));
		}
	}

	// Keep track of ALL conflicts
	// (even those across multiple attributes)
	let mut conflicting_ids = Vec::new();

	// Now, create instances for every attribute we got.
	for attr in all_attrs.into_iter() {
		let value = attributes
			.iter()
			.find(|(a_id, _)| *a_id == attr.id)
			.map(|x| &x.1);

		if let Some(value) = value {
			// Make sure type matches
			if value.as_stub() != attr.data_type {
				return Err(AddItemError::AttributeDataTypeMismatch.into());
			}

			let value_ser = serde_json::to_string(&value).unwrap();

			// Check "unique" constraint
			// Has no effect on blobs, so don't check them.
			// (this is why that switch is hidden in ui)
			if attr.options.is_unique && value.as_stub() != AttrDataStub::Blob {
				// Look for non-unique row
				match sqlx::query(
					"
					SELECT id FROM attribute_instance
					WHERE attribute_id=$1
					AND attribute_value=$2
					",
				)
				.bind(i64::from(attr.id))
				.bind(&value_ser)
				.fetch_all(&mut **t)
				.await
				{
					Ok(res) => {
						for row in res {
							conflicting_ids.push(row.get::<i64, _>("id").into())
						}
					}

					Err(e) => return Err(e.into()),
				};
			}

			// Create the attribute instances
			let res = sqlx::query(
				"INSERT INTO attribute_instance (item_id, attribute_id, attribute_value)
			VALUES ($1, $2, $3);",
			)
			.bind(i64::from(new_item))
			.bind(i64::from(attr.id))
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
		} else {
			// Check "not null" constraint
			if attr.options.is_not_null {
				return Err(AddItemError::NotNullViolated.into());
			}
		}
	}

	if conflicting_ids.len() != 0 {
		return Err(AddItemError::UniqueViolated { conflicting_ids }.into());
	}

	return Ok(new_item);
}
