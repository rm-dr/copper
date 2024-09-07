//! Helper structs that contain database element properties

use serde::{Deserialize, Serialize};
use smartstring::{LazyCompact, SmartString};
use utoipa::ToSchema;

use super::{
	data::AttrDataStub,
	handles::{AttributeId, DatasetId, ItemclassId},
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

/// Itemclass information
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ItemclassInfo {
	/// The id of the itemclass
	#[schema(value_type = u32)]
	pub id: ItemclassId,

	/// This itemclass' name
	#[schema(value_type = String)]
	pub name: SmartString<LazyCompact>,
}

/// Attribute information
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AttributeInfo {
	/// The id of this attribute
	#[schema(value_type = u32)]
	pub id: AttributeId,

	/// The itemclass this attribute belongs to
	#[schema(value_type = u32)]
	pub itemclass: ItemclassId,

	/// The order of this attribute in its itemclass.
	/// These start at 0, and must be unique & consecutive
	/// inside any itemclass.
	pub order: u32,

	/// This attribute's name
	#[schema(value_type = String)]
	pub name: SmartString<LazyCompact>,

	/// The type of data stored in this attribute
	pub data_type: AttrDataStub,

	/// If true, this attribute must contain a value
	pub is_not_null: bool,

	/// If true, each item in this attribute's itemclass must
	/// have a unique value in this attribute
	pub is_unique: bool,
}
