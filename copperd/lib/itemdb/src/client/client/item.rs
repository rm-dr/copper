//! This modules contains Copper's itemdb client

use itertools::Itertools;
use sqlx::Row;
use std::collections::BTreeMap;
use thiserror::Error;

use crate::{
	AttrData, AttrDataStub, AttributeId, AttributeInfo, AttributeOptions, ClassId, ItemId,
};

use super::ItemdbClient;
use crate::{
	client::errors::item::{CountItemsError, GetItemError, ListItemsError},
	ItemInfo,
};

/// An error we can encounter when creating an item
#[derive(Debug, Error)]
pub enum AddItemError {
	/// Database error
	#[error("database backend error")]
	DbError(#[from] sqlx::Error),

	/// We tried to add an item to a class that doesn't exist
	#[error("tried to add an item to a class that doesn't exist")]
	NoSuchClass,

	/// We tried to create an item that contains an
	/// attribute that doesn't exist
	#[error("tried to create an item an attribute that doesn't exist")]
	BadAttribute,

	/// We tried to create an item,
	/// but provided multiple values for one attribute
	#[error("multiple values were provided for one attribute")]
	RepeatedAttribute,

	/// We tried to assign data to an attribute,
	/// but that data has the wrong type
	#[error("tried to assign data to an attribute, but type doesn't match")]
	AttributeDataTypeMismatch,

	/// We tried to create an item that contains an
	/// attribute from another class
	#[error("tried to create an item with a foreign attribute")]
	ForeignAttribute,

	/// We tried to create an item with attribute that violate a "not null" constraint
	#[error("tried to create an item with attributes that violate a `not null` constraint")]
	NotNullViolated,

	/// We tried to create an item with attribute that violate a "unique" constraint
	#[error("tried to create an item with attributes that violate a `unique` constraint")]
	UniqueViolated { conflicting_ids: Vec<ItemId> },
}

impl ItemdbClient {
	//
	// MARK: crud
	//

	pub async fn get_item(
		&self,
		t: &mut sqlx::Transaction<'_, sqlx::Postgres>,
		item: ItemId,
	) -> Result<ItemInfo, GetItemError> {
		let (id, class): (ItemId, ClassId) = {
			let res = sqlx::query("SELECT * FROM item WHERE id=$1;")
				.bind(i64::from(item))
				.fetch_one(&mut **t)
				.await;

			match res {
				Err(sqlx::Error::RowNotFound) => return Err(GetItemError::NotFound),
				Err(e) => return Err(e.into()),
				Ok(res) => (
					res.get::<i64, _>("id").into(),
					res.get::<i64, _>("class_id").into(),
				),
			}
		};

		let mut attribute_values: BTreeMap<AttributeId, AttrData> = BTreeMap::new();

		// Fill in attributes that have data
		// Empty attributes will be `None`.
		let res = sqlx::query("SELECT * FROM attribute_instance WHERE item_id=$1;")
			.bind(i64::from(item))
			.fetch_all(&mut **t)
			.await?;
		for row in res {
			let attr_id: AttributeId = row.get::<i64, _>("attribute_id").into();
			let value: AttrData =
				serde_json::from_str(row.get::<&str, _>("attribute_value")).unwrap();

			let x = attribute_values.insert(attr_id, value);
			assert!(x.is_none()) // Each insert should be new
		}

		Ok(ItemInfo {
			id,
			class,
			attribute_values,
		})
	}

	pub async fn list_items(
		&self,
		t: &mut sqlx::Transaction<'_, sqlx::Postgres>,
		class: ClassId,
		skip: i64,
		count: usize,
	) -> Result<Vec<ItemInfo>, ListItemsError> {
		let res = sqlx::query(
			"
			SELECT * FROM attribute_instance
			WHERE item_id in (
				SELECT id FROM item
				WHERE class_id=$1
				ORDER BY id
				OFFSET $2 LIMIT $3
			)
			ORDER BY item_id;
			",
		)
		.bind(i64::from(class))
		.bind(skip)
		.bind(i64::try_from(count).unwrap())
		.fetch_all(&mut **t)
		.await;
		// Produces three columns:
		// item_id, attribute_id, attribute_value

		return match res {
			Err(sqlx::Error::RowNotFound) => Err(ListItemsError::ClassNotFound),
			Err(e) => Err(e.into()),
			Ok(rows) => {
				let mut out: BTreeMap<ItemId, ItemInfo> = BTreeMap::new();

				for row in rows {
					let item_id: ItemId = row.get::<i64, _>("item_id").into();
					let attr_id: AttributeId = row.get::<i64, _>("attribute_id").into();
					let value: AttrData =
						serde_json::from_str(row.get::<&str, _>("attribute_value")).unwrap();

					out.entry(item_id).or_insert_with(|| ItemInfo {
						id: item_id,
						class,
						attribute_values: BTreeMap::new(),
					});

					let x = out.get_mut(&item_id).unwrap();
					x.attribute_values.insert(attr_id, value);
				}

				Ok(out.into_values().collect())
			}
		};
	}

	pub async fn add_item(
		&self,
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

		// Create the new item, we attach attributes afterwards
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
				return Err(AddItemError::ForeignAttribute);
			}

			// ...and that we only saw one instance of each attribute
			if attributes.iter().filter(|(x, _)| x == attr_id).count() != 1 {
				return Err(AddItemError::RepeatedAttribute);
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
					return Err(AddItemError::AttributeDataTypeMismatch);
				}

				let value_ser = serde_json::to_string(&value).unwrap();

				// Check "unique" constraint
				// Has no effect on blobs, so don't check them.
				// (this is why that switch is hidden in ui)
				if attr.options.is_unique && value.as_stub() != AttrDataStub::Blob {
					// Look for non-unique row
					match sqlx::query(
						"
						SELECT item_id FROM attribute_instance
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
								conflicting_ids.push(row.get::<i64, _>("item_id").into())
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
							return Err(AddItemError::BadAttribute);
						} else {
							return Err(sqlx::Error::Database(e).into());
						}
					}
					Err(e) => return Err(e.into()),
				};
			} else {
				// Check "not null" constraint
				if attr.options.is_not_null {
					return Err(AddItemError::NotNullViolated);
				}
			}
		}

		if !conflicting_ids.is_empty() {
			return Err(AddItemError::UniqueViolated { conflicting_ids });
		}

		return Ok(new_item);
	}

	//
	// MARK: misc
	//

	pub async fn count_items(
		&self,
		t: &mut sqlx::Transaction<'_, sqlx::Postgres>,
		class: ClassId,
	) -> Result<i64, CountItemsError> {
		let res = sqlx::query("SELECT COUNT(*) FROM item WHERE class_id=$1;")
			.bind(i64::from(class))
			.fetch_one(&mut **t)
			.await;

		return match res {
			Err(sqlx::Error::RowNotFound) => Err(CountItemsError::ClassNotFound),
			Err(e) => Err(e.into()),
			Ok(res) => Ok(res.get("count")),
		};
	}
}
