//! This modules contains Copper's itemdb client

use copper_util::names::check_name;
use sqlx::Row;

use crate::{
	client::errors::attribute::{
		AddAttributeError, DeleteAttributeError, GetAttributeError, RenameAttributeError,
	},
	AttrDataStub, AttributeId, AttributeInfo, AttributeOptions, ClassId,
};

use super::ItemdbClient;

impl ItemdbClient {
	pub async fn add_attribute(
		&self,
		t: &mut sqlx::Transaction<'_, sqlx::Postgres>,
		in_class: ClassId,
		name: &str,
		with_type: AttrDataStub,
		options: AttributeOptions,
	) -> Result<AttributeId, AddAttributeError> {
		match check_name(name) {
			Ok(()) => {}
			Err(e) => return Err(AddAttributeError::NameError(e)),
		}

		// If we're trying to create a notnull attribute,
		// we need to ensure that no new null fields are created.
		// ...in other words, we must make sure that no items exist.
		if options.is_not_null {
			let res = sqlx::query("SELECT COUNT(id) FROM item WHERE class_id=$1;")
				.bind(i64::from(in_class))
				.fetch_one(&mut **t)
				.await?;
			let item_count = res.get::<i64, _>("count");

			if item_count != 0 {
				return Err(AddAttributeError::CreatedNotNullWhenItemsExist);
			}
		}

		let res = sqlx::query(
			"INSERT INTO attribute(class_id, attr_order, pretty_name, data_type, is_unique, is_not_null)
			SELECT $1, COALESCE(MAX(attr_order) + 1 , 0), $2, $3, $4, $5 FROM attribute WHERE class_id=$6
			RETURNING id;",
		)
		.bind(i64::from(in_class))
		.bind(name)
		.bind(serde_json::to_string(&with_type).unwrap())
		.bind(options.is_unique)
		.bind(options.is_not_null)
		.bind(i64::from(in_class))
		.fetch_one(&mut **t)
		.await;

		let new_handle: AttributeId = match res {
			Ok(res) => res.get::<i64, _>("id").into(),
			Err(sqlx::Error::Database(e)) => {
				if e.is_foreign_key_violation() {
					return Err(AddAttributeError::NoSuchClass);
				} else if e.is_unique_violation() {
					return Err(AddAttributeError::UniqueViolation);
				} else {
					return Err(sqlx::Error::Database(e).into());
				}
			}
			Err(e) => return Err(e.into()),
		};

		return Ok(new_handle);
	}

	pub async fn get_attribute(
		&self,
		t: &mut sqlx::Transaction<'_, sqlx::Postgres>,
		attribute: AttributeId,
	) -> Result<AttributeInfo, GetAttributeError> {
		let res = sqlx::query("SELECT * FROM attribute WHERE id=$1;")
			.bind(i64::from(attribute))
			.fetch_one(&mut **t)
			.await;

		return match res {
			Err(sqlx::Error::RowNotFound) => Err(GetAttributeError::NotFound),
			Err(e) => Err(e.into()),
			Ok(res) => Ok(AttributeInfo {
				id: res.get::<i64, _>("id").into(),
				class: res.get::<i64, _>("class_id").into(),
				order: res.get::<i64, _>("attr_order"),
				name: res.get::<String, _>("pretty_name").into(),
				data_type: serde_json::from_str(res.get::<&str, _>("data_type")).unwrap(),
				options: AttributeOptions {
					is_unique: res.get("is_unique"),
					is_not_null: res.get("is_not_null"),
				},
			}),
		};
	}

	pub async fn rename_attribute(
		&self,
		t: &mut sqlx::Transaction<'_, sqlx::Postgres>,
		attribute: AttributeId,
		new_name: &str,
	) -> Result<(), RenameAttributeError> {
		match check_name(new_name) {
			Ok(()) => {}
			Err(e) => return Err(RenameAttributeError::NameError(e)),
		}

		let res = sqlx::query("UPDATE attribute SET pretty_name=$1 WHERE id=$2;")
			.bind(new_name)
			.bind(i64::from(attribute))
			.execute(&mut **t)
			.await;

		return match res {
			Ok(_) => Ok(()),

			Err(sqlx::Error::Database(e)) => {
				if e.is_unique_violation() {
					Err(RenameAttributeError::UniqueViolation)
				} else {
					Err(sqlx::Error::Database(e).into())
				}
			}

			Err(e) => Err(e.into()),
		};
	}

	pub async fn del_attribute(
		&self,
		t: &mut sqlx::Transaction<'_, sqlx::Postgres>,
		attribute: AttributeId,
	) -> Result<(), DeleteAttributeError> {
		// This also deletes all attribute entries, etc,
		// since they're marked with ON DELETE CASCADE.
		sqlx::query("DELETE FROM attribute WHERE id=$1;")
			.bind(i64::from(attribute))
			.execute(&mut **t)
			.await?;

		return Ok(());
	}
}
