//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.15

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "item")]
pub struct Model {
	#[sea_orm(primary_key)]
	pub id: i32,
	pub class: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
	#[sea_orm(
		belongs_to = "super::class::Entity",
		from = "Column::Class",
		to = "super::class::Column::Id",
		on_update = "NoAction",
		on_delete = "NoAction"
	)]
	Class,
	#[sea_orm(has_many = "super::value_binary::Entity")]
	ValueBinary,
	#[sea_orm(has_many = "super::value_str::Entity")]
	ValueStr,
}

impl Related<super::class::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Class.def()
	}
}

impl Related<super::value_binary::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::ValueBinary.def()
	}
}

impl Related<super::value_str::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::ValueStr.def()
	}
}

impl ActiveModelBehavior for ActiveModel {}