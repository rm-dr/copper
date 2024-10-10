use copper_storaged::{AttrData, AttrDataStub, ClassId, ItemId};
use copper_util::HashType;
use serde::{Deserialize, Serialize};
use smartstring::{LazyCompact, SmartString};
use std::fmt::Debug;
use utoipa::ToSchema;

/// Attribute data, provided by the user by api calls.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type")]
pub enum ApiAttrData {
	/// Typed, unset data
	None { data_type: AttrDataStub },

	/// A block of text
	Text {
		#[schema(value_type = String)]
		value: SmartString<LazyCompact>,
	},

	/// An integer
	Integer {
		/// The integer
		value: i64,

		/// If true, this integer must be non-negative
		is_non_negative: bool,
	},

	/// A float
	Float {
		/// The float
		value: f64,

		/// If true, this float must be non-negative
		is_non_negative: bool,
	},

	/// A boolean
	Boolean { value: bool },

	/// A checksum
	Hash {
		/// The type of this hash
		hash_type: HashType,

		/// The hash data
		data: Vec<u8>,
	},

	/// Binary data stored in S3
	Blob {
		/// The object's key
		#[schema(value_type = String)]
		key: SmartString<LazyCompact>,
	},

	/// A reference to an item in another class
	Reference {
		/// The item class this reference points to
		#[schema(value_type = i64)]
		class: ClassId,

		/// The item
		#[schema(value_type = i64)]
		item: ItemId,
	},
}

impl TryFrom<&ApiAttrData> for AttrData {
	type Error = ();

	fn try_from(value: &ApiAttrData) -> Result<Self, Self::Error> {
		value.clone().try_into()
	}
}

impl TryFrom<ApiAttrData> for AttrData {
	type Error = ();

	fn try_from(value: ApiAttrData) -> Result<Self, Self::Error> {
		Ok(match value {
			ApiAttrData::Blob { .. } => return Err(()),

			ApiAttrData::None { data_type } => Self::None {
				data_type: data_type.clone(),
			},
			ApiAttrData::Boolean { value } => Self::Boolean { value },
			ApiAttrData::Text { value } => Self::Text { value },
			ApiAttrData::Hash { hash_type, data } => Self::Hash { hash_type, data },
			ApiAttrData::Reference { class, item } => Self::Reference { class, item },

			ApiAttrData::Float {
				value,
				is_non_negative,
			} => Self::Float {
				value,
				is_non_negative,
			},

			ApiAttrData::Integer {
				value,
				is_non_negative,
			} => Self::Integer {
				value,
				is_non_negative,
			},
		})
	}
}
