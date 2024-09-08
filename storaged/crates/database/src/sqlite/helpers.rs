use crate::api::{
	data::AttrData,
	errors::transaction::AddItemError,
	handles::{AttributeId, ClassId, ItemId},
};
use std::collections::BTreeMap;

pub(super) async fn add_item(
	t: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
	to_class: ClassId,
	attributes: BTreeMap<AttributeId, AttrData>,
) -> Result<ItemId, AddItemError> {
	let res = sqlx::query("INSERT INTO item(class_id) VALUES ?;")
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
		match sqlx::query("SELECT FROM attribute WHERE id=? AND class_id=?;")
			.fetch_one(&mut **t)
			.await
		{
			Ok(_) => {}
			Err(sqlx::Error::RowNotFound) => return Err(AddItemError::ForeignAttribute),
			Err(e) => return Err(AddItemError::DbError(Box::new(e))),
		}

		// Create the attribute instance
		let res = sqlx::query(
			"INSERT INTO attribute_instance(item_id, attribute_id, attribute_value)
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
