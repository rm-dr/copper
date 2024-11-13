use copper_util::names::check_name;
use sqlx::Row;

use crate::{
	client::errors::dataset::{
		AddDatasetError, DeleteDatasetError, GetDatasetError, ListDatasetsError, RenameDatasetError,
	},
	AttributeInfo, AttributeOptions, ClassId, ClassInfo, DatasetId, DatasetInfo, UserId,
};

use super::ItemdbClient;

impl ItemdbClient {
	pub async fn add_dataset(
		&self,
		t: &mut sqlx::Transaction<'_, sqlx::Postgres>,
		name: &str,
		user: UserId,
	) -> Result<DatasetId, AddDatasetError> {
		match check_name(name) {
			Ok(()) => {}
			Err(e) => return Err(AddDatasetError::NameError(e)),
		}

		let res =
			sqlx::query("INSERT INTO dataset (pretty_name, owner) VALUES ($1, $2) RETURNING id;")
				.bind(name)
				.bind(i64::from(user))
				.fetch_one(&mut **t)
				.await;

		let new_handle: DatasetId = match res {
			Ok(res) => res.get::<i64, _>("id").into(),
			Err(sqlx::Error::Database(e)) => {
				if e.is_unique_violation() {
					return Err(AddDatasetError::UniqueViolation);
				} else {
					return Err(sqlx::Error::Database(e).into());
				}
			}
			Err(e) => return Err(e.into()),
		};

		return Ok(new_handle);
	}

	pub async fn get_dataset(
		&self,
		t: &mut sqlx::Transaction<'_, sqlx::Postgres>,
		dataset: DatasetId,
	) -> Result<DatasetInfo, GetDatasetError> {
		let classes: Vec<ClassInfo> = {
			let rows = sqlx::query("SELECT * FROM class WHERE dataset_id=$1;")
				.bind(i64::from(dataset))
				.fetch_all(&mut **t)
				.await?;

			let mut classes = Vec::new();

			for r in rows {
				let class_id: ClassId = r.get::<i64, _>("id").into();

				let attr_rows = sqlx::query("SELECT * FROM attribute WHERE class_id=$1;")
					.bind(i64::from(class_id))
					.fetch_all(&mut **t)
					.await?;

				let attributes = attr_rows
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
					.bind(i64::from(class_id))
					.fetch_one(&mut **t)
					.await?;

				let item_count = res.get::<i64, _>("count").try_into().unwrap();

				classes.push(ClassInfo {
					dataset,
					id: class_id,
					name: r.get::<String, _>("pretty_name").into(),
					attributes,
					item_count,
				});
			}

			classes
		};

		let res = sqlx::query("SELECT * FROM dataset WHERE id=$1;")
			.bind(i64::from(dataset))
			.fetch_one(&mut **t)
			.await;

		return match res {
			Err(sqlx::Error::RowNotFound) => Err(GetDatasetError::NotFound),
			Err(e) => Err(e.into()),
			Ok(res) => Ok(DatasetInfo {
				id: res.get::<i64, _>("id").into(),
				owner: res.get::<i64, _>("owner").into(),
				name: res.get::<String, _>("pretty_name").into(),
				classes,
			}),
		};
	}

	pub async fn list_datasets(
		&self,
		t: &mut sqlx::Transaction<'_, sqlx::Postgres>,
		owner: UserId,
	) -> Result<Vec<DatasetInfo>, ListDatasetsError> {
		let rows = sqlx::query("SELECT * FROM dataset WHERE owner=$1;")
			.bind(i64::from(owner))
			.fetch_all(&mut **t)
			.await?;

		let mut out = Vec::new();
		for row in rows {
			let dataset_id = row.get::<i64, _>("id").into();

			let classes: Vec<ClassInfo> = {
				let rows = sqlx::query("SELECT * FROM class WHERE dataset_id=$1;")
					.bind(i64::from(dataset_id))
					.fetch_all(&mut **t)
					.await?;

				let mut classes = Vec::new();

				for r in rows {
					let class_id: ClassId = r.get::<i64, _>("id").into();

					let attr_rows = sqlx::query("SELECT * FROM attribute WHERE class_id=$1;")
						.bind(i64::from(class_id))
						.fetch_all(&mut **t)
						.await?;

					let attributes = attr_rows
						.into_iter()
						.map(|row| AttributeInfo {
							id: row.get::<i64, _>("id").into(),
							class: row.get::<i64, _>("id").into(),
							order: row.get::<i64, _>("attr_order"),
							name: row.get::<String, _>("pretty_name").into(),
							data_type: serde_json::from_str(row.get::<&str, _>("data_type"))
								.unwrap(),
							options: AttributeOptions {
								is_unique: row.get("is_unique"),
								is_not_null: row.get("is_not_null"),
							},
						})
						.collect();

					let res = sqlx::query("SELECT COUNT(id) FROM item WHERE class_id=$1;")
						.bind(i64::from(class_id))
						.fetch_one(&mut **t)
						.await?;

					let item_count = res.get::<i64, _>("count").try_into().unwrap();

					classes.push(ClassInfo {
						dataset: dataset_id,
						id: class_id,
						name: r.get::<String, _>("pretty_name").into(),
						attributes,
						item_count,
					});
				}

				classes
			};

			out.push(DatasetInfo {
				id: dataset_id,
				owner: row.get::<i64, _>("owner").into(),
				name: row.get::<String, _>("pretty_name").into(),
				classes,
			});
		}

		return Ok(out);
	}

	pub async fn rename_dataset(
		&self,
		t: &mut sqlx::Transaction<'_, sqlx::Postgres>,
		dataset: DatasetId,
		new_name: &str,
	) -> Result<(), RenameDatasetError> {
		match check_name(new_name) {
			Ok(()) => {}
			Err(e) => return Err(RenameDatasetError::NameError(e)),
		}

		let res = sqlx::query("UPDATE dataset SET pretty_name=$1 WHERE id=$2;")
			.bind(new_name)
			.bind(i64::from(dataset))
			.execute(&mut **t)
			.await;

		return match res {
			Ok(_) => Ok(()),
			Err(sqlx::Error::Database(e)) => {
				if e.is_unique_violation() {
					Err(RenameDatasetError::UniqueViolation)
				} else {
					Err(sqlx::Error::Database(e).into())
				}
			}
			Err(e) => Err(e.into()),
		};
	}

	pub async fn del_dataset(
		&self,
		t: &mut sqlx::Transaction<'_, sqlx::Postgres>,
		dataset: DatasetId,
	) -> Result<(), DeleteDatasetError> {
		// This also deletes all attributes, etc,
		// since they're marked with ON DELETE CASCADE.
		sqlx::query("DELETE FROM dataset WHERE id=$1;")
			.bind(i64::from(dataset))
			.execute(&mut **t)
			.await?;

		return Ok(());
	}
}
