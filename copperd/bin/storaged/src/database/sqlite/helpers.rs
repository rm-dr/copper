use itertools::Itertools;
use sqlx::Row;

use crate::database::base::{
	data::{AttrData, AttrDataStub},
	errors::transaction::AddItemError,
	handles::{AttributeId, ClassId, ItemId},
};

pub(super) async fn add_item(
	t: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
	to_class: ClassId,
	attributes: Vec<(AttributeId, AttrData)>,
) -> Result<ItemId, AddItemError> {
	// Make sure we have at most one of each attribute
	if !attributes.iter().map(|(x, _)| x).all_unique() {
		return Err(AddItemError::RepeatedAttribute);
	}

	let res = sqlx::query("INSERT INTO item (class_id) VALUES (?);")
		.bind(u32::from(to_class))
		.execute(&mut **t)
		.await;

	let new_item: ItemId = match res {
		Ok(x) => u32::try_from(x.last_insert_rowid()).unwrap().into(),
		Err(sqlx::Error::Database(e)) => {
			if e.is_foreign_key_violation() {
				return Err(AddItemError::NoSuchClass);
			} else {
				let e = Box::new(sqlx::Error::Database(e));
				return Err(AddItemError::DbError(e));
			}
		}
		Err(e) => return Err(AddItemError::DbError(Box::new(e))),
	};

	for (attr, value) in attributes {
		// Make sure this attribute comes from this class
		let data_type =
			match sqlx::query("SELECT data_type FROM attribute WHERE id=? AND class_id=?;")
				.bind(u32::from(attr))
				.bind(u32::from(to_class))
				.fetch_one(&mut **t)
				.await
			{
				Ok(res) => {
					let data_type: AttrDataStub =
						serde_json::from_str(&res.get::<String, _>("data_type")).unwrap();

					data_type
				}
				Err(sqlx::Error::RowNotFound) => return Err(AddItemError::ForeignAttribute),
				Err(e) => return Err(AddItemError::DbError(Box::new(e))),
			};

		// Make sure type matches
		if data_type != value.to_stub() {
			return Err(AddItemError::AttributeDataTypeMismatch);
		}

		// Create the attribute instance
		let res = sqlx::query(
			"INSERT INTO attribute_instance (item_id, attribute_id, attribute_value)
			VALUES (?, ?, ?);",
		)
		.bind(u32::from(new_item))
		.bind(u32::from(attr))
		.bind(serde_json::to_string(&value).unwrap())
		.execute(&mut **t)
		.await;

		match res {
			Ok(_) => {}
			Err(sqlx::Error::Database(e)) => {
				if e.is_foreign_key_violation() {
					return Err(AddItemError::BadAttribute);
				} else {
					let e = Box::new(sqlx::Error::Database(e));
					return Err(AddItemError::DbError(e));
				}
			}
			Err(e) => return Err(AddItemError::DbError(Box::new(e))),
		};
	}

	return Ok(new_item);
}
