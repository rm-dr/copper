use copper_storaged::{AttrData, AttrDataStub, AttributeId, ClassId, ItemId};
use itertools::Itertools;
use sqlx::Row;

use crate::database::base::errors::transaction::AddItemError;

pub(super) async fn add_item(
	t: &mut sqlx::Transaction<'_, sqlx::Postgres>,
	to_class: ClassId,
	attributes: Vec<(AttributeId, AttrData)>,
) -> Result<ItemId, AddItemError> {
	// Make sure we have at most one of each attribute
	if !attributes.iter().map(|(x, _)| x).all_unique() {
		return Err(AddItemError::RepeatedAttribute);
	}

	let res = sqlx::query("INSERT INTO item (class_id) VALUES ($1) RETURNING id;")
		.bind(i64::from(to_class))
		.fetch_one(&mut **t)
		.await;

	let new_item: ItemId = match res {
		Ok(res) => res.get::<i64, _>("id").into(),
		Err(sqlx::Error::Database(e)) => {
			if e.is_foreign_key_violation() {
				return Err(AddItemError::NoSuchClass);
			} else {
				return Err(sqlx::Error::Database(e).into());
			}
		}
		Err(e) => return Err(e.into()),
	};

	for (attr, value) in attributes {
		// Make sure this attribute comes from this class
		let data_type =
			match sqlx::query("SELECT data_type FROM attribute WHERE id=$1 AND class_id=$2;")
				.bind(i64::from(attr))
				.bind(i64::from(to_class))
				.fetch_one(&mut **t)
				.await
			{
				Ok(res) => {
					let data_type: AttrDataStub =
						serde_json::from_str(&res.get::<String, _>("data_type")).unwrap();

					data_type
				}
				Err(sqlx::Error::RowNotFound) => return Err(AddItemError::ForeignAttribute),
				Err(e) => return Err(e.into()),
			};

		// Make sure type matches
		if data_type != value.as_stub() {
			return Err(AddItemError::AttributeDataTypeMismatch);
		}

		// Create the attribute instance
		let res = sqlx::query(
			"INSERT INTO attribute_instance (item_id, attribute_id, attribute_value)
			VALUES ($1, $2, $3);",
		)
		.bind(i64::from(new_item))
		.bind(i64::from(attr))
		.bind(serde_json::to_string(&value).unwrap())
		.execute(&mut **t)
		.await;

		match res {
			Ok(_) => {}
			Err(sqlx::Error::Database(e)) => {
				if e.is_foreign_key_violation() {
					return Err(AddItemError::BadAttribute);
				} else {
					return Err(sqlx::Error::Database(e).into());
				}
			}
			Err(e) => return Err(e.into()),
		};
	}

	return Ok(new_item);
}
