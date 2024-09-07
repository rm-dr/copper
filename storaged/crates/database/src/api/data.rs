use copper_util::mime::MimeType;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use smartstring::{LazyCompact, SmartString};
use std::{fmt::Debug, sync::Arc};
use utoipa::ToSchema;

use super::handles::{ClassHandle, ItemIdx};

/// Bits of data inside a metadata db.
#[derive(Debug, Clone)]
pub enum DatasetData {
	/// Typed, unset data
	None(DatasetDataStub),

	/// A block of text
	Text(Arc<SmartString<LazyCompact>>),

	/// An integer
	Integer { value: i64, is_non_negative: bool },

	/// A float
	Float { value: f64, is_non_negative: bool },

	/// A boolean
	Boolean(bool),

	/// A checksum
	Hash {
		format: HashType,
		data: Arc<Vec<u8>>,
	},

	/// Binary data stored in S3
	Blob {
		/// This data's media type
		mime: MimeType,

		/// The data
		url: String,
	},

	Reference {
		/// The item class this reference points to
		class: ClassHandle,

		/// The item
		item: ItemIdx,
	},
}

impl DatasetData {
	pub fn is_none(&self) -> bool {
		matches!(self, Self::None(_))
	}

	pub fn is_blob(&self) -> bool {
		matches!(self, Self::Blob { .. })
	}

	/// Convert a hash to a hex string
	pub fn hash_to_string(data: &[u8]) -> String {
		data.iter().map(|x| format!("{:02X}", x)).join("")
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, ToSchema)]
pub enum HashType {
	MD5,
	SHA256,
	SHA512,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type")]
pub enum DatasetDataStub {
	/// Plain text
	Text,

	/// Binary data, in any format
	Blob,

	/// An integer
	Integer { is_non_negative: bool },

	/// A float
	Float { is_non_negative: bool },

	/// A boolean
	Boolean,

	/// A checksum
	Hash { hash_type: HashType },

	/// A reference to an item
	Reference {
		#[schema(value_type = u32)]
		class: ClassHandle,
	},
}
