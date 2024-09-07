//! Types and instances of data we can store in an attribute

use copper_util::mime::MimeType;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use smartstring::{LazyCompact, SmartString};
use std::{fmt::Debug, sync::Arc};
use utoipa::ToSchema;

use super::handles::{ItemIdx, ItemclassId};

/// A value stored inside an attribute.
/// Each of these corresponds to an [`AttrDataStub`]
#[derive(Debug, Clone)]
pub enum AttrData {
	/// Typed, unset data
	None(AttrDataStub),

	/// A block of text
	Text(Arc<SmartString<LazyCompact>>),

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
	Boolean(bool),

	/// A checksum
	Hash {
		/// The type of this hash
		format: HashType,

		/// The hash data
		data: Vec<u8>,
	},

	/// Binary data stored in S3
	Blob {
		/// This data's media type
		mime: MimeType,

		/// The data
		url: String,
	},

	/// A reference to an item in another class
	Reference {
		/// The item class this reference points to
		class: ItemclassId,

		/// The item
		item: ItemIdx,
	},
}

impl AttrData {
	/// Is this `Self::None`?
	pub fn is_none(&self) -> bool {
		matches!(self, Self::None(_))
	}

	/// Is this `Self::Blob`?
	pub fn is_blob(&self) -> bool {
		matches!(self, Self::Blob { .. })
	}

	/// Convert a hash to a hex string
	pub fn hash_to_string(data: &[u8]) -> String {
		data.iter().map(|x| format!("{:02X}", x)).join("")
	}
}

/// The types of hashes we support
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, ToSchema)]
#[allow(missing_docs)]
pub enum HashType {
	MD5,
	SHA256,
	SHA512,
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
		/// The itemclass we reference
		#[schema(value_type = u32)]
		class: ItemclassId,
	},
}
