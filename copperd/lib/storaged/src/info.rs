//! Helper structs that contain database element properties

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use smartstring::{LazyCompact, SmartString};
use utoipa::ToSchema;

use super::{
	data::{AttrData, AttrDataStub},
	id::{AttributeId, ClassId, DatasetId, ItemId},
};

/// Dataset information
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DatasetInfo {
	/// The id of this dataset
	#[schema(value_type = u32)]
	pub id: DatasetId,

	/// This dataset's name
	#[schema(value_type = String)]
	pub name: SmartString<LazyCompact>,
}

/// Class information
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ClassInfo {
	/// The id of the class
	#[schema(value_type = u32)]
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
	#[schema(value_type = u32)]
	pub id: AttributeId,

	/// The class this attribute belongs to
	#[schema(value_type = u32)]
	pub class: ClassId,

	/// The order of this attribute in its class.
	/// These start at 0, and must be unique & consecutive
	/// inside any class.
	pub order: u32,

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
	#[schema(value_type = u32)]
	pub id: ItemId,

	/// The class this item belongs to
	#[schema(value_type = u32)]
	pub class: ClassId,

	/// All attributes this item has
	pub attribute_values: BTreeMap<AttributeId, AttrData>,
}
