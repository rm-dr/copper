//! Types and instances of data we can store in an attribute

use copper_util::HashType;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use smartstring::{LazyCompact, SmartString};
use std::fmt::Debug;
use utoipa::ToSchema;

use super::id::{ClassId, ItemId};

/// A value stored inside an attribute.
/// These are never directly provided by users,
/// and are serialized as json in the item db.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AttrData {
	/// A block of text
	Text { value: SmartString<LazyCompact> },

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
		/// The name of the bucket this blob is stored in
		bucket: SmartString<LazyCompact>,

		/// The object's key
		key: SmartString<LazyCompact>,
	},

	/// A reference to an item in another class
	Reference {
		/// The item class this reference points to
		class: ClassId,

		/// The item
		item: ItemId,
	},
}

impl AttrData {
	/// Is this `Self::Blob`?
	pub fn is_blob(&self) -> bool {
		matches!(self, Self::Blob { .. })
	}

	/// Convert a hash to a hex string
	pub fn hash_to_string(data: &[u8]) -> String {
		data.iter().map(|x| format!("{:02X}", x)).join("")
	}

	/// Convert this data instance to its type
	pub fn as_stub(&self) -> AttrDataStub {
		match self {
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
		#[schema(value_type = i64)]
		class: ClassId,
	},
}
