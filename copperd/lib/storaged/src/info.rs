//! Helper structs that contain database element properties

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use smartstring::{LazyCompact, SmartString};
use utoipa::ToSchema;

use crate::UserId;

use super::{
	data::{AttrData, AttrDataStub},
	id::{AttributeId, ClassId, DatasetId, ItemId},
};

#[derive(Debug, Deserialize, Serialize, ToSchema, Default)]
/// Options we can set when creating an attribute
pub struct AttributeOptions {
	/// If true, this attribute must have a value
	pub is_not_null: bool,

	/// If true, this attribute must be unique within its column
	pub unique: bool,
}

/// Dataset information
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DatasetInfo {
	/// The id of this dataset
	#[schema(value_type = i64)]
	pub id: DatasetId,

	/// The id of the user that owns this dataset
	#[schema(value_type = i64)]
	pub owner: UserId,

	/// This dataset's name
	#[schema(value_type = String)]
	pub name: SmartString<LazyCompact>,

	/// This dataset's classes
	pub classes: Vec<ClassInfo>,
}

/// Class information
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ClassInfo {
	/// The dataset this class is in
	#[schema(value_type = i64)]
	pub dataset: DatasetId,

	/// The id of the class
	#[schema(value_type = i64)]
	pub id: ClassId,

	/// This class' name
	#[schema(value_type = String)]
	pub name: SmartString<LazyCompact>,

	pub attributes: Vec<AttributeInfo>,
}

/// Attribute information
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AttributeInfo {
	/// The id of this attribute
	#[schema(value_type = i64)]
	pub id: AttributeId,

	/// The class this attribute belongs to
	#[schema(value_type = i64)]
	pub class: ClassId,

	/// The order of this attribute in its class.
	/// These start at 0, and must be unique & consecutive
	/// inside any class.
	pub order: i64,

	/// This attribute's name
	#[schema(value_type = String)]
	pub name: SmartString<LazyCompact>,

	/// The type of data stored in this attribute
	pub data_type: AttrDataStub,

	/// If true, this attribute must contain a value
	pub is_not_null: bool,

	/// If true, each item in this attribute's class must
	/// have a unique value in this attribute
	pub is_unique: bool,
}

/// Item information
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ItemInfo {
	/// The id of this item
	#[schema(value_type = i64)]
	pub id: ItemId,

	/// The class this item belongs to
	#[schema(value_type = i64)]
	pub class: ClassId,

	/// All attributes this item has
	pub attribute_values: BTreeMap<AttributeId, AttrData>,
}
