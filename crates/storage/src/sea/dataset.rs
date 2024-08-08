use futures::executor::block_on;
use sea_orm::{
	ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, Database, DatabaseConnection,
	DbBackend, DbErr, EntityTrait, QueryFilter, QuerySelect, Statement,
};
use sea_orm_migration::prelude::*;

use super::{
	entities::{prelude::*, *},
	errors::SeaDatasetError,
	migrator::Migrator,
};
use crate::{
	api::{AttrHandle, AttributeOptions, ClassHandle, Dataset, ItemHandle},
	StorageData, StorageDataType,
};

pub struct SeaDataset {
	database_url: String,
	database_name: String,

	conn: Option<DatabaseConnection>,
}

impl SeaDataset {
	pub fn new(database_url: &str, database_name: &str) -> Self {
		Self {
			database_name: database_name.into(),
			database_url: database_url.into(),
			conn: None,
		}
	}

	pub fn connect(&mut self) -> Result<(), DbErr> {
		if self.conn.is_some() {
			return Ok(());
		}
		let conn = block_on(Database::connect(&self.database_url))?;

		let conn = match conn.get_database_backend() {
			DbBackend::MySql => {
				block_on(conn.execute(Statement::from_string(
					conn.get_database_backend(),
					format!("CREATE DATABASE IF NOT EXISTS `{}`;", &self.database_name),
				)))?;

				let url = format!("{}/{}", &self.database_url, &self.database_name);
				block_on(Database::connect(&url))?
			}
			DbBackend::Postgres => {
				block_on(conn.execute(Statement::from_string(
					conn.get_database_backend(),
					format!("DROP DATABASE IF EXISTS \"{}\";", &self.database_name),
				)))?;
				block_on(conn.execute(Statement::from_string(
					conn.get_database_backend(),
					format!("CREATE DATABASE \"{}\";", &self.database_name),
				)))?;

				let url = format!("{}/{}", &self.database_url, &self.database_name);
				block_on(Database::connect(&url))?
			}
			DbBackend::Sqlite => conn,
		};

		// TODO: this destroys data. Don't do that if you don't have to.
		block_on(Migrator::refresh(&conn))?;
		self.conn = Some(conn);
		return Ok(());
	}
}

impl Dataset for SeaDataset {
	fn add_attr(
		&mut self,
		class: ClassHandle,
		name: &str,
		data_type: StorageDataType,
		options: AttributeOptions,
	) -> Result<AttrHandle, ()> {
		let new_attr = attr::ActiveModel {
			id: ActiveValue::NotSet,
			name: ActiveValue::Set(name.into()),
			class: ActiveValue::Set(usize::from(class).try_into().unwrap()),
			datatype: ActiveValue::Set(
				match data_type {
					StorageDataType::Text => "string",
					StorageDataType::Binary => "binary",
				}
				.into(),
			),
			is_unique: ActiveValue::Set(options.unique),
		};

		let res = block_on(
			Attr::insert(new_attr).exec(
				self.conn
					.as_mut()
					.ok_or(SeaDatasetError::NotConnected)
					.unwrap(),
			),
		)
		.map_err(SeaDatasetError::Database)
		.unwrap();

		Ok(usize::try_from(res.last_insert_id).unwrap().into())
	}

	fn add_class(&mut self, name: &str) -> Result<ClassHandle, ()> {
		let new_class = class::ActiveModel {
			id: ActiveValue::NotSet,
			name: ActiveValue::Set(name.into()),
		};
		let res = block_on(
			Class::insert(new_class).exec(
				self.conn
					.as_mut()
					.ok_or(SeaDatasetError::NotConnected)
					.unwrap(),
			),
		)
		.map_err(SeaDatasetError::Database)
		.unwrap();

		Ok(usize::try_from(res.last_insert_id).unwrap().into())
	}

	fn add_item(&mut self, class: ClassHandle) -> Result<ItemHandle, ()> {
		let new_item = item::ActiveModel {
			id: ActiveValue::NotSet,
			class: ActiveValue::Set(usize::from(class).try_into().unwrap()),
		};
		let res = block_on(
			Item::insert(new_item).exec(
				self.conn
					.as_mut()
					.ok_or(SeaDatasetError::NotConnected)
					.unwrap(),
			),
		)
		.map_err(SeaDatasetError::Database)
		.unwrap();

		Ok(usize::try_from(res.last_insert_id).unwrap().into())
	}
	fn add_item_with_attrs(
		&mut self,
		_class: ClassHandle,
		_attrs: &[&StorageData],
	) -> Result<ItemHandle, ()> {
		unimplemented!()
	}

	fn del_attr(&mut self, _attr: AttrHandle) -> Result<(), ()> {
		unimplemented!()
	}

	fn del_class(&mut self, _class: ClassHandle) -> Result<(), ()> {
		unimplemented!()
	}

	fn del_item(&mut self, _item: ItemHandle) -> Result<(), ()> {
		unimplemented!()
	}

	fn get_attr(&self, attr_name: &str) -> Result<Option<AttrHandle>, ()> {
		let found_attr: Option<attr::Model> = block_on(
			Attr::find()
				.filter(attr::Column::Name.eq(attr_name))
				//.select_only()
				//.columns([attr::Column::Id, attr::Column::Name])
				.one(
					self.conn
						.as_ref()
						.ok_or(SeaDatasetError::NotConnected)
						.unwrap(),
				),
		)
		.map_err(SeaDatasetError::Database)
		.unwrap();
		return Ok(found_attr.map(|x| usize::try_from(x.id).unwrap().into()));
	}

	fn get_class(&self, class_name: &str) -> Result<Option<ClassHandle>, ()> {
		let found_class: Option<class::Model> = block_on(
			Class::find()
				.select_only()
				.columns([class::Column::Id, class::Column::Name])
				.filter(class::Column::Name.eq(class_name))
				.one(
					self.conn
						.as_ref()
						.ok_or(SeaDatasetError::NotConnected)
						.unwrap(),
				),
		)
		.map_err(SeaDatasetError::Database)
		.unwrap();
		return Ok(found_class.map(|x| usize::try_from(x.id).unwrap().into()));
	}

	/*
	fn iter_items(&self) -> Result<impl Iterator<Item = ItemHandle>, ()> {
		unimplemented!();
		Ok([].iter().cloned())
	}

	fn iter_attrs(&self) -> Result<impl Iterator<Item = AttrHandle>, ()> {
		unimplemented!();
		Ok([].iter().cloned())
	}

	fn iter_classes(&self) -> Result<impl Iterator<Item = ClassHandle>, ()> {
		unimplemented!();
		Ok([].iter().cloned())
	}
	*/

	/*
	TODO: Bug?
		fn iter_classes(&self) -> impl Iterator<Item = ClassHandle> {
		unimplemented!();
		[].iter().cloned()
	} */

	fn item_get_attr(&self, _item: ItemHandle, _attr: AttrHandle) -> Result<StorageData, ()> {
		unimplemented!()
	}

	fn item_get_class(&self, _item: ItemHandle) -> Result<ClassHandle, ()> {
		unimplemented!()
	}

	fn item_set_attr(
		&mut self,
		item: ItemHandle,
		attr: AttrHandle,
		data: &StorageData,
	) -> Result<(), ()> {
		if self.attr_get_type(attr)? != data.get_type() {
			return Err(());
			//return Err(SeaDatasetError::TypeMismatch);
		}

		match data {
			StorageData::None(t) => {
				match t {
					StorageDataType::Text => {
						block_on(
							value_str::Entity::delete_many()
								.filter(value_str::Column::Attr.eq(
									<i32 as TryFrom<usize>>::try_from(usize::from(attr)).unwrap(),
								))
								.filter(value_str::Column::Attr.eq(
									<i32 as TryFrom<usize>>::try_from(usize::from(item)).unwrap(),
								))
								.exec(
									self.conn
										.as_ref()
										.ok_or(SeaDatasetError::NotConnected)
										.unwrap(),
								),
						)
						.map_err(SeaDatasetError::Database)
						.unwrap();
					}
					StorageDataType::Binary => {
						block_on(
							value_binary::Entity::delete_many()
								.filter(value_binary::Column::Attr.eq(
									<i32 as TryFrom<usize>>::try_from(usize::from(attr)).unwrap(),
								))
								.filter(value_binary::Column::Attr.eq(
									<i32 as TryFrom<usize>>::try_from(usize::from(item)).unwrap(),
								))
								.exec(
									self.conn
										.as_ref()
										.ok_or(SeaDatasetError::NotConnected)
										.unwrap(),
								),
						)
						.map_err(SeaDatasetError::Database)
						.unwrap();
					}
				}
			}
			StorageData::Binary { data, format } => {
				let new_value = value_binary::ActiveModel {
					id: ActiveValue::NotSet,
					attr: ActiveValue::Set(usize::from(attr).try_into().unwrap()),
					item: ActiveValue::Set(usize::from(item).try_into().unwrap()),
					value: ActiveValue::Set((**data).clone()),
					format: ActiveValue::Set(format.to_string()),
				};
				let _res = block_on(
					ValueBinary::insert(new_value)
						.on_conflict(
							OnConflict::columns([value_str::Column::Attr, value_str::Column::Item])
								.update_column(value_str::Column::Value)
								.to_owned(),
						)
						.exec(
							self.conn
								.as_mut()
								.ok_or(SeaDatasetError::NotConnected)
								.unwrap(),
						),
				)
				.map_err(SeaDatasetError::Database)
				.unwrap();
			}
			StorageData::Text(text) => {
				let found_attr: Option<attr::Model> = block_on(
					Attr::find_by_id(<i32 as TryFrom<usize>>::try_from(usize::from(attr)).unwrap())
						.one(
							self.conn
								.as_ref()
								.ok_or(SeaDatasetError::NotConnected)
								.unwrap(),
						),
				)
				.map_err(SeaDatasetError::Database)
				.unwrap();

				if found_attr.is_none() {
					return Err(());
					//return Err(SeaDatasetError::BadAttrHandle);
				}

				if found_attr.unwrap().is_unique {
					let found_val: Option<value_str::Model> =
						block_on(
							ValueStr::find()
								.filter(value_str::Column::Attr.eq(
									<i32 as TryFrom<usize>>::try_from(usize::from(attr)).unwrap(),
								))
								.filter(value_str::Column::Value.eq((**text).clone()))
								.one(
									self.conn
										.as_ref()
										.ok_or(SeaDatasetError::NotConnected)
										.unwrap(),
								),
						)
						.map_err(SeaDatasetError::Database)
						.unwrap();
					if found_val.is_some() {
						return Err(());
						//return Err(SeaDatasetError::UniqueViolated);
					}
				}

				let new_value = value_str::ActiveModel {
					id: ActiveValue::NotSet,
					attr: ActiveValue::Set(usize::from(attr).try_into().unwrap()),
					item: ActiveValue::Set(usize::from(item).try_into().unwrap()),
					value: ActiveValue::Set((**text).clone()),
				};
				let _res = block_on(
					ValueStr::insert(new_value)
						.on_conflict(
							OnConflict::columns([value_str::Column::Attr, value_str::Column::Item])
								.update_column(value_str::Column::Value)
								.to_owned(),
						)
						.exec(
							self.conn
								.as_mut()
								.ok_or(SeaDatasetError::NotConnected)
								.unwrap(),
						),
				)
				.map_err(SeaDatasetError::Database)
				.unwrap();
			}
		};

		Ok(())
	}

	fn class_set_name(&mut self, _class: ClassHandle, _name: &str) -> Result<(), ()> {
		unimplemented!()
	}

	fn class_get_name(&self, _class: ClassHandle) -> Result<&str, ()> {
		unimplemented!()
	}

	/*
		fn class_get_attrs(&self, _class: ClassHandle) -> Result<impl Iterator<Item = AttrHandle>, ()> {
			unimplemented!();
			Ok([].iter().cloned())
		}
	*/
	fn class_num_attrs(&self, _class: ClassHandle) -> Result<usize, ()> {
		unimplemented!()
	}

	fn attr_set_name(&mut self, _attr: AttrHandle, _name: &str) -> Result<(), ()> {
		unimplemented!()
	}

	fn attr_get_name(&self, _attr: AttrHandle) -> Result<&str, ()> {
		unimplemented!()
	}

	fn attr_get_type(&self, attr: AttrHandle) -> Result<StorageDataType, ()> {
		let attr: Option<attr::Model> = block_on(
			Attr::find_by_id(<i32 as TryFrom<usize>>::try_from(usize::from(attr)).unwrap()).one(
				self.conn
					.as_ref()
					.ok_or(SeaDatasetError::NotConnected)
					.unwrap(),
			),
		)
		.map_err(SeaDatasetError::Database)
		.unwrap();

		if attr.is_none() {
			return Err(());
			//return Err(SeaDatasetError::BadAttrHandle);
		}

		// TODO: improve
		let t = match &attr.unwrap().datatype[..] {
			"string" => StorageDataType::Text,
			"binary" => StorageDataType::Binary,
			x => unreachable!("Bad type {x}"),
		};
		return Ok(t);
	}

	fn attr_get_class(&self, _attr: AttrHandle) -> ClassHandle {
		unimplemented!()
	}
}

/*
	let schema_manager = SchemaManager::new(db); // To investigate the schema
	assert!(schema_manager.has_table("item").await?);
	assert!(schema_manager.has_table("attr").await?);
	// Finding all
	let bakeries: Vec<bakery::Model> = Bakery::find().all(db).await?;
	assert_eq!(bakeries.len(), 1);
	// Finding by id
	let sad_bakery: Option<bakery::Model> = Bakery::find_by_id(1).one(db).await?;
	assert_eq!(sad_bakery.unwrap().name, "Sad Bakery");
*/
