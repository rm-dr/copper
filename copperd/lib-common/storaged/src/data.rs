//! Types and instances of data we can store in an attribute

use copper_util::{mime::MimeType, HashType};
use itertools::Itertools;
use pipelined_node_base::data::{CopperData, CopperDataStub};
use serde::{Deserialize, Serialize};
use smartstring::{LazyCompact, SmartString};
use std::fmt::Debug;
use utoipa::ToSchema;

use super::id::{ClassId, ItemId};

/// A value stored inside an attribute.
/// Each of these corresponds to an [`AttrDataStub`]
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type")]
pub enum AttrData {
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
		/// This data's media type
		#[schema(value_type = String)]
		mime: MimeType,

		/// The data
		url: String,
	},

	/// A reference to an item in another class
	Reference {
		/// The item class this reference points to
		#[schema(value_type = u32)]
		class: ClassId,

		/// The item
		#[schema(value_type = u32)]
		item: ItemId,
	},
}

impl AttrData {
	/// Is this `Self::None`?
	pub fn is_none(&self) -> bool {
		matches!(self, Self::None { .. })
	}

	/// Is this `Self::Blob`?
	pub fn is_blob(&self) -> bool {
		matches!(self, Self::Blob { .. })
	}

	/// Convert a hash to a hex string
	pub fn hash_to_string(data: &[u8]) -> String {
		data.iter().map(|x| format!("{:02X}", x)).join("")
	}

	/// Convert this data instance to its type
	pub fn to_stub(&self) -> AttrDataStub {
		match self {
			Self::None { data_type } => data_type.clone(),
			Self::Blob { .. } => AttrDataStub::Blob,
			Self::Boolean { .. } => AttrDataStub::Boolean,
			Self::Text { .. } => AttrDataStub::Text,

			Self::Float {
				is_non_negative, ..
			} => AttrDataStub::Float {
				is_non_negative: *is_non_negative,
			},

			Self::Integer {
				is_non_negative, ..
			} => AttrDataStub::Integer {
				is_non_negative: *is_non_negative,
			},

			Self::Hash { hash_type, .. } => AttrDataStub::Hash {
				hash_type: *hash_type,
			},

			Self::Reference { class, .. } => AttrDataStub::Reference { class: *class },
		}
	}
}

impl TryFrom<CopperData> for AttrData {
	type Error = ();

	fn try_from(value: CopperData) -> Result<Self, Self::Error> {
		return Ok(match value {
			CopperData::None { data_type } => Self::None {
				data_type: data_type.into(),
			},

			CopperData::Blob { .. } => return Err(()),
			CopperData::Text { value } => Self::Text { value },
			CopperData::Boolean { value } => Self::Boolean { value },
			CopperData::Hash { hash_type, data } => Self::Hash { hash_type, data },

			CopperData::Reference { class, item } => Self::Reference {
				class: class.into(),
				item: item.into(),
			},

			CopperData::Float {
				value,
				is_non_negative,
			} => Self::Float {
				value,
				is_non_negative,
			},

			CopperData::Integer {
				value,
				is_non_negative,
			} => Self::Integer {
				value,
				is_non_negative,
			},
		});
	}
}

impl TryInto<CopperData> for AttrData {
	type Error = ();

	fn try_into(self) -> Result<CopperData, Self::Error> {
		return Ok(match self {
			Self::None { data_type } => CopperData::None {
				data_type: data_type.into(),
			},

			Self::Blob { .. } => return Err(()),
			Self::Text { value } => CopperData::Text { value },
			Self::Boolean { value } => CopperData::Boolean { value },
			Self::Hash { hash_type, data } => CopperData::Hash { hash_type, data },

			Self::Reference { class, item } => CopperData::Reference {
				class: class.into(),
				item: item.into(),
			},

			Self::Float {
				value,
				is_non_negative,
			} => CopperData::Float {
				value,
				is_non_negative,
			},

			Self::Integer {
				value,
				is_non_negative,
			} => CopperData::Integer {
				value,
				is_non_negative,
			},
		});
	}
}

/// The type of data stored in an attribute.
/// Each of these corresponds to a variant of [`AttrData`]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type")]
pub enum AttrDataStub {
	/// Plain text
	Text,

	/// Binary data, in any format
	Blob,

	/// An integer
	Integer {
		/// If true, this integer must be non-negative
		is_non_negative: bool,
	},

	/// A float
	Float {
		/// If true, this float must be non-negative
		is_non_negative: bool,
	},

	/// A boolean
	Boolean,

	/// A checksum
	Hash {
		/// The type of this hash
		hash_type: HashType,
	},

	/// A reference to an item
	Reference {
		/// The class we reference
		#[schema(value_type = u32)]
		class: ClassId,
	},
}

impl Into<CopperDataStub> for AttrDataStub {
	fn into(self) -> CopperDataStub {
		match self {
			Self::Text => CopperDataStub::Text,
			Self::Blob => CopperDataStub::Blob,
			Self::Integer { is_non_negative } => CopperDataStub::Integer { is_non_negative },
			Self::Float { is_non_negative } => CopperDataStub::Integer { is_non_negative },
			Self::Boolean => CopperDataStub::Boolean,
			Self::Hash { hash_type } => CopperDataStub::Hash { hash_type },
			Self::Reference { class } => CopperDataStub::Reference {
				class: u32::from(class),
			},
		}
	}
}

impl From<CopperDataStub> for AttrDataStub {
	fn from(value: CopperDataStub) -> Self {
		match value {
			CopperDataStub::Text => Self::Text,
			CopperDataStub::Blob => Self::Blob,
			CopperDataStub::Integer { is_non_negative } => Self::Integer { is_non_negative },
			CopperDataStub::Float { is_non_negative } => Self::Integer { is_non_negative },
			CopperDataStub::Boolean => Self::Boolean,
			CopperDataStub::Hash { hash_type } => Self::Hash { hash_type },
			CopperDataStub::Reference { class } => Self::Reference {
				class: class.into(),
			},
		}
	}
}
