use migrator::Migrator;
use sea_orm::{
	ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, Database, DatabaseConnection,
	DbBackend, DbErr, EntityTrait, QueryFilter, Statement,
};
use sea_orm_migration::prelude::*;

mod entities;
mod migrator;

use entities::{prelude::*, *};
use ufo_util::data::{PipelineData, PipelineDataType};

use crate::api::{Dataset, DatasetHandle};

use self::migrator::AttrDatatype;

// TODO: split this file
// TODO: easy way to make db for dev to generate entities

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SeaItemIdx(i32);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SeaClassIdx(i32);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SeaAttrIdx(i32);
impl DatasetHandle for SeaItemIdx {}
impl DatasetHandle for SeaClassIdx {}
impl DatasetHandle for SeaAttrIdx {}

impl From<i32> for SeaItemIdx {
	fn from(value: i32) -> Self {
		Self(value)
	}
}

impl From<SeaItemIdx> for i32 {
	fn from(value: SeaItemIdx) -> Self {
		value.0
	}
}

impl From<i32> for SeaAttrIdx {
	fn from(value: i32) -> Self {
		Self(value)
	}
}

impl From<SeaAttrIdx> for i32 {
	fn from(value: SeaAttrIdx) -> Self {
		value.0
	}
}

impl From<i32> for SeaClassIdx {
	fn from(value: i32) -> Self {
		Self(value)
	}
}

impl From<SeaClassIdx> for i32 {
	fn from(value: SeaClassIdx) -> Self {
		value.0
	}
}

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

	pub async fn connect(&mut self) -> Result<(), DbErr> {
		if self.conn.is_some() {
			return Ok(());
		}
		let conn = Database::connect(&self.database_url).await?;

		let conn = match conn.get_database_backend() {
			DbBackend::MySql => {
				conn.execute(Statement::from_string(
					conn.get_database_backend(),
					format!("CREATE DATABASE IF NOT EXISTS `{}`;", &self.database_name),
				))
				.await?;

				let url = format!("{}/{}", &self.database_url, &self.database_name);
				Database::connect(&url).await?
			}
			DbBackend::Postgres => {
				conn.execute(Statement::from_string(
					conn.get_database_backend(),
					format!("DROP DATABASE IF EXISTS \"{}\";", &self.database_name),
				))
				.await?;
				conn.execute(Statement::from_string(
					conn.get_database_backend(),
					format!("CREATE DATABASE \"{}\";", &self.database_name),
				))
				.await?;

				let url = format!("{}/{}", &self.database_url, &self.database_name);
				Database::connect(&url).await?
			}
			DbBackend::Sqlite => conn,
		};

		// TODO: this destroys data. Don't do that if you don't have to.
		Migrator::refresh(&conn).await?;
		self.conn = Some(conn);
		return Ok(());
	}
}

impl Dataset for SeaDataset {
	type ClassHandle = SeaClassIdx;
	type AttrHandle = SeaAttrIdx;
	type ItemHandle = SeaItemIdx;
	type ErrorType = DbErr;

	async fn add_attr(
		&mut self,
		class: Self::ClassHandle,
		name: &str,
		data_type: ufo_util::data::PipelineDataType,
	) -> Result<Self::AttrHandle, Self::ErrorType> {
		let new_attr = attr::ActiveModel {
			id: ActiveValue::NotSet,
			name: ActiveValue::Set(name.into()),
			class: ActiveValue::set(class.into()),
			datatype: ActiveValue::Set(
				match data_type {
					PipelineDataType::Text => AttrDatatype::String,
					PipelineDataType::Binary => unimplemented!(),
				}
				.to_string(),
			),
		};
		let res = Attr::insert(new_attr)
			.exec(self.conn.as_mut().unwrap())
			.await?;

		Ok(res.last_insert_id.into())
	}

	async fn add_class(&mut self, name: &str) -> Result<Self::ClassHandle, Self::ErrorType> {
		let new_class = class::ActiveModel {
			id: ActiveValue::NotSet,
			name: ActiveValue::Set(name.into()),
		};
		let res = Class::insert(new_class)
			.exec(self.conn.as_mut().unwrap())
			.await?;

		Ok(res.last_insert_id.into())
	}

	async fn add_item(
		&mut self,
		class: Self::ClassHandle,
	) -> Result<Self::ItemHandle, Self::ErrorType> {
		let new_item = item::ActiveModel {
			id: ActiveValue::NotSet,
			class: ActiveValue::Set(class.into()),
		};
		let res = Item::insert(new_item)
			.exec(self.conn.as_mut().unwrap())
			.await?;

		Ok(res.last_insert_id.into())
	}
	async fn add_item_with_attrs(
		&mut self,
		_class: Self::ClassHandle,
		_attrs: &[&PipelineData],
	) -> Result<Self::ItemHandle, Self::ErrorType> {
		unimplemented!()
	}

	async fn del_attr(&mut self, _attr: Self::AttrHandle) -> Result<(), Self::ErrorType> {
		unimplemented!()
	}

	async fn del_class(&mut self, _class: Self::ClassHandle) -> Result<(), Self::ErrorType> {
		unimplemented!()
	}

	async fn del_item(&mut self, _item: Self::ItemHandle) -> Result<(), Self::ErrorType> {
		unimplemented!()
	}

	async fn get_attr(&self, attr_name: &str) -> Option<Self::AttrHandle> {
		let found_attr: Option<attr::Model> = Attr::find()
			.filter(attr::Column::Name.eq(attr_name))
			.one(self.conn.as_ref().unwrap())
			.await
			.unwrap();
		return Some(found_attr.unwrap().id.into());
	}

	async fn get_class(&self, class_name: &str) -> Option<Self::ClassHandle> {
		let found_class: Option<class::Model> = Class::find()
			.filter(class::Column::Name.eq(class_name))
			.one(self.conn.as_ref().unwrap())
			.await
			.unwrap();
		return Some(found_class.unwrap().id.into());
	}

	async fn iter_items(&self) -> impl Iterator<Item = Self::ItemHandle> {
		unimplemented!();
		[].iter().cloned()
	}

	async fn iter_attrs(&self) -> impl Iterator<Item = Self::AttrHandle> {
		unimplemented!();
		[].iter().cloned()
	}

	async fn iter_classes(&self) -> impl Iterator<Item = Self::ClassHandle> {
		unimplemented!();
		[].iter().cloned()
	}

	/*
	TODO: Bug?
		fn iter_classes(&self) -> impl Iterator<Item = Self::ClassHandle> {
		unimplemented!();
		[].iter().cloned()
	} */

	async fn item_get_attr(
		&self,
		_item: Self::ItemHandle,
		_attr: Self::AttrHandle,
	) -> Result<PipelineData, Self::ErrorType> {
		unimplemented!()
	}

	async fn item_get_class(&self, _item: Self::ItemHandle) -> Self::ClassHandle {
		unimplemented!()
	}

	async fn item_set_attr(
		&mut self,
		item: Self::ItemHandle,
		attr: Self::AttrHandle,
		data: &PipelineData,
	) -> Result<(), Self::ErrorType> {
		// TODO: check type

		match data {
			PipelineData::None(t) => match t {
				PipelineDataType::Text => {
					let existing_data: Option<value_str::Model> = ValueStr::find()
						.filter(value_str::Column::Attr.eq(Into::<i32>::into(attr)))
						.filter(value_str::Column::Item.eq(Into::<i32>::into(item)))
						.one(self.conn.as_ref().unwrap())
						.await?;
					if let Some(x) = existing_data {
						let del_value = value_str::ActiveModel {
							id: ActiveValue::Set(x.id),
							..Default::default()
						};
						del_value.delete(self.conn.as_ref().unwrap()).await?;
					}
				}
				PipelineDataType::Binary => {}
			},
			PipelineData::Binary { .. } => {}
			PipelineData::Text(text) => {
				let existing_data: Option<value_str::Model> = ValueStr::find()
					.filter(value_str::Column::Attr.eq(Into::<i32>::into(attr)))
					.filter(value_str::Column::Item.eq(Into::<i32>::into(item)))
					.one(self.conn.as_ref().unwrap())
					.await?;

				if let Some(x) = existing_data {
					let new_value = value_str::ActiveModel {
						id: ActiveValue::Set(x.id),
						attr: ActiveValue::Unchanged(attr.into()),
						item: ActiveValue::Unchanged(item.into()),
						value: ActiveValue::Set(text.clone()),
					};
					new_value.update(self.conn.as_ref().unwrap()).await?;
				} else {
					let new_value = value_str::ActiveModel {
						id: ActiveValue::NotSet,
						attr: ActiveValue::Set(attr.into()),
						item: ActiveValue::Set(item.into()),
						value: ActiveValue::Set(text.clone()),
					};
					let _res = ValueStr::insert(new_value)
						.exec(self.conn.as_mut().unwrap())
						.await?;
				}
			}
		};

		Ok(())
	}

	async fn class_set_name(
		&mut self,
		_class: Self::ClassHandle,
		_name: &str,
	) -> Result<(), Self::ErrorType> {
		unimplemented!()
	}

	async fn class_get_name(&self, _class: Self::ClassHandle) -> &str {
		unimplemented!()
	}

	async fn class_get_attrs(
		&self,
		_class: Self::ClassHandle,
	) -> impl Iterator<Item = Self::AttrHandle> {
		unimplemented!();
		[].iter().cloned()
	}

	async fn class_num_attrs(&self, _class: Self::ClassHandle) -> usize {
		unimplemented!()
	}

	async fn attr_set_name(
		&mut self,
		_attr: Self::AttrHandle,
		_name: &str,
	) -> Result<(), Self::ErrorType> {
		unimplemented!()
	}

	async fn attr_get_name(&self, _attr: Self::AttrHandle) -> &str {
		unimplemented!()
	}

	async fn attr_get_type(&self, _attr: Self::AttrHandle) -> PipelineDataType {
		unimplemented!()
	}

	async fn attr_get_class(&self, _attr: Self::AttrHandle) -> Self::ClassHandle {
		unimplemented!()
	}
}

pub async fn run() -> Result<(), DbErr> {
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

	Ok(())
}
