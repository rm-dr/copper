use copper_util::mime::MimeType;
use serde::{Deserialize, Serialize};
use smartstring::{LazyCompact, SmartString};
use std::{fmt::Debug, sync::Arc};
use url::Url;
use utoipa::ToSchema;

use crate::base::{PipelineData, PipelineDataStub};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, ToSchema)]
pub enum HashType {
	MD5,
	SHA256,
	SHA512,
}

/// Immutable bits of data inside a pipeline.
///
/// Cloning [`CopperData`] should be very fast. Consider wrapping
/// big containers in an [`Arc`].
///
/// Any variant that has a "deserialize" implementation
/// may be used as a parameter in certain nodes.
/// (for example, the `Constant` node's `value` field)
///
/// This is very similar to [`MetastoreData`]. In fact, we often convert between the two.
/// We can't use [`MetastoreData`] everywhere, though... Data inside a pipeline is represented
/// slightly differently than data inside a metastore. (For example, look at the `Blob` variant.
/// In a metastore, `Blob`s are always stored in a blobstore. Here, they are given as streams.)
///
/// Also, some types that exist here cannot exist inside a metastore (for example, `Path`, which
/// represents a file path that is available when the pipeline is run. This path may vanish later.)
#[derive(Debug, Clone, Deserialize, ToSchema)]
#[serde(tag = "data_type")]
pub enum CopperData {
	/// Typed, unset data
	#[serde(skip)]
	None { data_type: CopperDataStub },

	/// A block of text
	Text {
		#[schema(value_type = String)]
		value: Arc<SmartString<LazyCompact>>,
	},

	/// An integer
	Integer { value: i64, is_non_negative: bool },

	/// A boolean
	Boolean { value: bool },

	/// A float
	Float { value: f64, is_non_negative: bool },

	/// A checksum
	#[serde(skip)]
	Hash {
		hash_type: HashType,
		data: Arc<Vec<u8>>,
	},

	/// Arbitrary binary data.
	/// This will be stored in the metadata db.
	#[serde(skip)]
	Bytes {
		/// This data's media type
		mime: MimeType,

		/// The data
		source: BytesSource,
	},

	/// TODO: Use ClassId here?
	/// would have to expose storaged lib...
	Reference {
		/// The item's class
		class: u32,

		/// The item
		item: u32,
	},
}

#[derive(Debug, Clone)]
pub enum BytesSource {
	Array {
		fragment: Arc<Vec<u8>>,
		is_last: bool,
	},
	Url {
		url: Url,
	},
}

impl PipelineData for CopperData {
	type DataStubType = CopperDataStub;

	fn as_stub(&self) -> Self::DataStubType {
		match self {
			Self::None { data_type } => *data_type,
			Self::Text { .. } => CopperDataStub::Text,
			Self::Integer {
				is_non_negative, ..
			} => CopperDataStub::Integer {
				is_non_negative: *is_non_negative,
			},
			Self::Boolean { .. } => CopperDataStub::Boolean,
			Self::Float {
				is_non_negative, ..
			} => CopperDataStub::Float {
				is_non_negative: *is_non_negative,
			},
			Self::Hash {
				hash_type: format, ..
			} => CopperDataStub::Hash { hash_type: *format },
			Self::Bytes { .. } => CopperDataStub::Bytes,
			Self::Reference { class, .. } => CopperDataStub::Reference { class: *class },
		}
	}
}

impl CopperData {
	pub fn is_none(&self) -> bool {
		matches!(self, Self::None { .. })
	}
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize, ToSchema)]
#[serde(tag = "stub_type")]
pub enum CopperDataStub {
	/// Plain text
	Text,

	/// A binary blob
	Bytes,

	/// An integer
	Integer { is_non_negative: bool },

	/// A float
	Float { is_non_negative: bool },

	/// A boolean
	Boolean,

	/// A checksum
	Hash { hash_type: HashType },

	/// A reference to an item
	Reference { class: u32 },
}

impl PipelineDataStub for CopperDataStub {
	fn is_subset_of(&self, superset: &Self) -> bool {
		if self == superset {
			return true;
		}

		return false;
	}
}
