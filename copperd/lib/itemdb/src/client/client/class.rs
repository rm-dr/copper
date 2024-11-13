//! This modules contains Copper's itemdb client

use copper_util::names::check_name;
use sqlx::Row;

use crate::{
	client::errors::class::{AddClassError, DeleteClassError, GetClassError, RenameClassError},
	AttributeInfo, AttributeOptions, ClassId, ClassInfo, DatasetId,
};

use super::ItemdbClient;

impl ItemdbClient {
	pub async fn add_class(
		&self,
		t: &mut sqlx::Transaction<'_, sqlx::Postgres>,
		in_dataset: DatasetId,
		name: &str,
	) -> Result<ClassId, AddClassError> {
		match check_name(name) {
			Ok(()) => {}
			Err(e) => return Err(AddClassError::NameError(e)),
		}

		let res = sqlx::query(
			"INSERT INTO class (dataset_id, pretty_name) VALUES ($1, $2) RETURNING id;",
		)
		.bind(i64::from(in_dataset))
		.bind(name)
		.fetch_one(&mut **t)
		.await;

		let new_handle: ClassId = match res {
			Ok(res) => res.get::<i64, _>("id").into(),
			Err(sqlx::Error::Database(e)) => {
				if e.is_foreign_key_violation() {
					return Err(AddClassError::NoSuchDataset);
				} else if e.is_unique_violation() {
					return Err(AddClassError::UniqueViolation);
				} else {
					return Err(sqlx::Error::Database(e).into());
				}
			}
			Err(e) => return Err(e.into()),
		};
		return Ok(new_handle);
	}

	pub async fn get_class(
		&self,
		t: &mut sqlx::Transaction<'_, sqlx::Postgres>,
		class: ClassId,
	) -> Result<ClassInfo, GetClassError> {
		let rows = sqlx::query("SELECT * FROM attribute WHERE class_id=$1;")
			.bind(i64::from(class))
			.fetch_all(&mut **t)
			.await?;

		let attributes = rows
			.into_iter()
			.map(|row| AttributeInfo {
				id: row.get::<i64, _>("id").into(),
				class: row.get::<i64, _>("id").into(),
				order: row.get::<i64, _>("attr_order"),
				name: row.get::<String, _>("pretty_name").into(),
				data_type: serde_json::from_str(row.get::<&str, _>("data_type")).unwrap(),
				options: AttributeOptions {
					is_unique: row.get("is_unique"),
					is_not_null: row.get("is_not_null"),
				},
			})
			.collect();

		let res = sqlx::query("SELECT COUNT(id) FROM item WHERE class_id=$1;")
			.bind(i64::from(class))
			.fetch_one(&mut **t)
			.await?;

		let item_count = res.get::<i64, _>("count").try_into().unwrap();

		let res = sqlx::query("SELECT * FROM class WHERE id=$1;")
			.bind(i64::from(class))
			.fetch_one(&mut **t)
			.await;

		return match res {
			Err(sqlx::Error::RowNotFound) => Err(GetClassError::NotFound),
			Err(e) => Err(e.into()),
			Ok(res) => Ok(ClassInfo {
				dataset: res.get::<i64, _>("dataset_id").into(),
				id: res.get::<i64, _>("id").into(),
				name: res.get::<String, _>("pretty_name").into(),
				attributes,
				item_count,
			}),
		};
	}

	pub async fn rename_class(
		&self,
		t: &mut sqlx::Transaction<'_, sqlx::Postgres>,
		class: ClassId,
		new_name: &str,
	) -> Result<(), RenameClassError> {
		match check_name(new_name) {
			Ok(()) => {}
			Err(e) => return Err(RenameClassError::NameError(e)),
		}

		let res = sqlx::query("UPDATE class SET pretty_name=$1 WHERE id=$2;")
			.bind(new_name)
			.bind(i64::from(class))
			.execute(&mut **t)
			.await;

		return match res {
			Err(sqlx::Error::Database(e)) => {
				if e.is_unique_violation() {
					Err(RenameClassError::UniqueViolation)
				} else {
					Err(sqlx::Error::Database(e).into())
				}
			}

			Err(e) => Err(e.into()),

			Ok(_) => Ok(()),
		};
	}

	pub async fn del_class(
		&self,
		t: &mut sqlx::Transaction<'_, sqlx::Postgres>,
		class: ClassId,
	) -> Result<(), DeleteClassError> {
		// This also deletes all classes, attributes, etc,
		// since they're marked with ON DELETE CASCADE.
		sqlx::query("DELETE FROM class WHERE id=$1;")
			.bind(i64::from(class))
			.execute(&mut **t)
			.await?;

		return Ok(());
	}
}
