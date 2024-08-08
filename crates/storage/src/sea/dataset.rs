use sea_orm::{
	ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, Database, DatabaseConnection,
	DbBackend, DbErr, EntityTrait, QueryFilter, QuerySelect, Statement,
};
use sea_orm_migration::prelude::*;
use ufo_util::data::{PipelineData, PipelineDataType};

use super::{
	entities::{prelude::*, *},
	errors::SeaDatasetError,
	migrator::Migrator,
};
use crate::api::{Dataset, DatasetHandle};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SeaItemHandle(i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SeaClassHandle(i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SeaAttrHandle(i32);

impl DatasetHandle for SeaItemHandle {}
impl DatasetHandle for SeaClassHandle {}
impl DatasetHandle for SeaAttrHandle {}

impl From<i32> for SeaItemHandle {
	fn from(value: i32) -> Self {
		Self(value)
	}
}

impl From<SeaItemHandle> for i32 {
	fn from(value: SeaItemHandle) -> Self {
		value.0
	}
}

impl From<i32> for SeaAttrHandle {
	fn from(value: i32) -> Self {
		Self(value)
	}
}

impl From<SeaAttrHandle> for i32 {
	fn from(value: SeaAttrHandle) -> Self {
		value.0
	}
}

impl From<i32> for SeaClassHandle {
	fn from(value: i32) -> Self {
		Self(value)
	}
}

impl From<SeaClassHandle> for i32 {
	fn from(value: SeaClassHandle) -> Self {
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
	type ClassHandle = SeaClassHandle;
	type AttrHandle = SeaAttrHandle;
	type ItemHandle = SeaItemHandle;
	type ErrorType = SeaDatasetError;

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
					PipelineDataType::Text => "string",
					PipelineDataType::Binary => "binary",
				}
				.into(),
			),
		};

		let res = Attr::insert(new_attr)
			.exec(self.conn.as_mut().ok_or(SeaDatasetError::NotConnected)?)
			.await
			.map_err(SeaDatasetError::Database)?;

		Ok(res.last_insert_id.into())
	}

	async fn add_class(&mut self, name: &str) -> Result<Self::ClassHandle, Self::ErrorType> {
		let new_class = class::ActiveModel {
			id: ActiveValue::NotSet,
			name: ActiveValue::Set(name.into()),
		};
		let res = Class::insert(new_class)
			.exec(self.conn.as_mut().ok_or(SeaDatasetError::NotConnected)?)
			.await
			.map_err(SeaDatasetError::Database)?;

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
			.exec(self.conn.as_mut().ok_or(SeaDatasetError::NotConnected)?)
			.await
			.map_err(SeaDatasetError::Database)?;

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

	async fn get_attr(&self, attr_name: &str) -> Result<Option<Self::AttrHandle>, Self::ErrorType> {
		let found_attr: Option<attr::Model> = Attr::find()
			.filter(attr::Column::Name.eq(attr_name))
			//.select_only()
			//.columns([attr::Column::Id, attr::Column::Name])
			.one(self.conn.as_ref().ok_or(SeaDatasetError::NotConnected)?)
			.await
			.map_err(SeaDatasetError::Database)?;
		return Ok(found_attr.map(|x| x.id.into()));
	}

	async fn get_class(
		&self,
		class_name: &str,
	) -> Result<Option<Self::ClassHandle>, Self::ErrorType> {
		let found_class: Option<class::Model> = Class::find()
			.select_only()
			.columns([class::Column::Id, class::Column::Name])
			.filter(class::Column::Name.eq(class_name))
			.one(self.conn.as_ref().ok_or(SeaDatasetError::NotConnected)?)
			.await
			.map_err(SeaDatasetError::Database)?;
		return Ok(found_class.map(|x| x.id.into()));
	}

	async fn iter_items(&self) -> Result<impl Iterator<Item = Self::ItemHandle>, Self::ErrorType> {
		unimplemented!();
		Ok([].iter().cloned())
	}

	async fn iter_attrs(&self) -> Result<impl Iterator<Item = Self::AttrHandle>, Self::ErrorType> {
		unimplemented!();
		Ok([].iter().cloned())
	}

	async fn iter_classes(
		&self,
	) -> Result<impl Iterator<Item = Self::ClassHandle>, Self::ErrorType> {
		unimplemented!();
		Ok([].iter().cloned())
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

	async fn item_get_class(
		&self,
		_item: Self::ItemHandle,
	) -> Result<Self::ClassHandle, Self::ErrorType> {
		unimplemented!()
	}

	async fn item_set_attr(
		&mut self,
		item: Self::ItemHandle,
		attr: Self::AttrHandle,
		data: &PipelineData,
	) -> Result<(), Self::ErrorType> {
		if self.attr_get_type(attr).await? != data.get_type() {
			return Err(SeaDatasetError::TypeMismatch);
		}

		match data {
			PipelineData::None(t) => match t {
				PipelineDataType::Text => {
					value_str::Entity::delete_many()
						.filter(value_str::Column::Attr.eq(Into::<i32>::into(attr)))
						.filter(value_str::Column::Attr.eq(Into::<i32>::into(item)))
						.exec(self.conn.as_ref().ok_or(SeaDatasetError::NotConnected)?)
						.await
						.map_err(SeaDatasetError::Database)?;
				}
				PipelineDataType::Binary => {
					value_binary::Entity::delete_many()
						.filter(value_binary::Column::Attr.eq(Into::<i32>::into(attr)))
						.filter(value_binary::Column::Attr.eq(Into::<i32>::into(item)))
						.exec(self.conn.as_ref().ok_or(SeaDatasetError::NotConnected)?)
						.await
						.map_err(SeaDatasetError::Database)?;
				}
			},
			PipelineData::Binary { data, format } => {
				let new_value = value_binary::ActiveModel {
					id: ActiveValue::NotSet,
					attr: ActiveValue::Set(attr.into()),
					item: ActiveValue::Set(item.into()),
					value: ActiveValue::Set((**data).clone()),
					format: ActiveValue::Set(format.to_string()),
				};
				let _res = ValueBinary::insert(new_value)
					.on_conflict(
						OnConflict::columns([value_str::Column::Attr, value_str::Column::Item])
							.update_column(value_str::Column::Value)
							.to_owned(),
					)
					.exec(self.conn.as_mut().ok_or(SeaDatasetError::NotConnected)?)
					.await
					.map_err(SeaDatasetError::Database)?;
			}
			PipelineData::Text(text) => {
				let new_value = value_str::ActiveModel {
					id: ActiveValue::NotSet,
					attr: ActiveValue::Set(attr.into()),
					item: ActiveValue::Set(item.into()),
					value: ActiveValue::Set((**text).clone()),
				};
				let _res = ValueStr::insert(new_value)
					.on_conflict(
						OnConflict::columns([value_str::Column::Attr, value_str::Column::Item])
							.update_column(value_str::Column::Value)
							.to_owned(),
					)
					.exec(self.conn.as_mut().ok_or(SeaDatasetError::NotConnected)?)
					.await
					.map_err(SeaDatasetError::Database)?;
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

	async fn class_get_name(&self, _class: Self::ClassHandle) -> Result<&str, Self::ErrorType> {
		unimplemented!()
	}

	async fn class_get_attrs(
		&self,
		_class: Self::ClassHandle,
	) -> Result<impl Iterator<Item = Self::AttrHandle>, Self::ErrorType> {
		unimplemented!();
		Ok([].iter().cloned())
	}

	async fn class_num_attrs(&self, _class: Self::ClassHandle) -> Result<usize, Self::ErrorType> {
		unimplemented!()
	}

	async fn attr_set_name(
		&mut self,
		_attr: Self::AttrHandle,
		_name: &str,
	) -> Result<(), Self::ErrorType> {
		unimplemented!()
	}

	async fn attr_get_name(&self, _attr: Self::AttrHandle) -> Result<&str, Self::ErrorType> {
		unimplemented!()
	}

	async fn attr_get_type(
		&self,
		attr: Self::AttrHandle,
	) -> Result<PipelineDataType, Self::ErrorType> {
		let attr: Option<attr::Model> = Attr::find_by_id(Into::<i32>::into(attr))
			.one(self.conn.as_ref().ok_or(SeaDatasetError::NotConnected)?)
			.await
			.map_err(SeaDatasetError::Database)?;

		if attr.is_none() {
			return Err(SeaDatasetError::BadAttrHandle);
		}

		// TODO: improve
		let t = match &attr.unwrap().datatype[..] {
			"string" => PipelineDataType::Text,
			"binary" => PipelineDataType::Binary,
			x => unreachable!("Bad type {x}"),
		};
		return Ok(t);
	}

	async fn attr_get_class(&self, _attr: Self::AttrHandle) -> Self::ClassHandle {
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
