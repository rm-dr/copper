use copper_ds_core::{
	data::{HashType, MetastoreData, MetastoreDataStub},
	handles::{ClassHandle, ItemIdx},
};
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, path::PathBuf, sync::Arc};
use copper_pipeline::api::{PipelineData, PipelineDataStub};
use copper_util::mime::MimeType;
use utoipa::ToSchema;

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
		value: Arc<String>,
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

	Reference {
		/// The item class this
		#[schema(value_type = u32)]
		class: ClassHandle,

		/// The item
		#[schema(value_type = u32)]
		item: ItemIdx,
	},
}

#[derive(Debug, Clone)]
pub enum BytesSource {
	Array {
		fragment: Arc<Vec<u8>>,
		is_last: bool,
	},
	File {
		path: PathBuf,
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

	fn disconnected(stub: Self::DataStubType) -> Self {
		Self::None { data_type: stub }
	}
}

impl CopperData {
	pub fn is_none(&self) -> bool {
		matches!(self, Self::None { .. })
	}

	pub fn as_db_data(&self) -> Option<MetastoreData> {
		Some(match self {
			// These may not be converted to MetastoreData directly.
			// - Blobs must first be written to the blobstore
			// - Paths may not be stored in a metastore at all.
			CopperData::Bytes { .. } => return None,

			CopperData::Text { value } => MetastoreData::Text(value.clone()),
			CopperData::Boolean { value } => MetastoreData::Boolean(*value),

			CopperData::None { data_type } => {
				if let Some(stub) = data_type.as_metastore_stub() {
					MetastoreData::None(stub)
				} else {
					return None;
				}
			}
			CopperData::Hash {
				hash_type: format,
				data,
			} => MetastoreData::Hash {
				format: *format,
				data: data.clone(),
			},
			CopperData::Integer {
				value,
				is_non_negative,
			} => MetastoreData::Integer {
				value: *value,
				is_non_negative: *is_non_negative,
			},
			CopperData::Float {
				value,
				is_non_negative,
			} => MetastoreData::Float {
				value: *value,
				is_non_negative: *is_non_negative,
			},
			CopperData::Reference { class, item } => MetastoreData::Reference {
				class: *class,
				item: *item,
			},
		})
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
	Reference {
		#[schema(value_type = u32)]
		class: ClassHandle,
	},
}

impl PipelineDataStub for CopperDataStub {
	fn is_subset_of(&self, superset: &Self) -> bool {
		if self == superset {
			return true;
		}

		return false;
	}
}

impl CopperDataStub {
	/// Get the [`MetastoreDataStub`] that this [`CopperDataStub`] encode to, if any.
	/// Not all [`CopperDataStub`]s may be stored in a metastore.
	fn as_metastore_stub(&self) -> Option<MetastoreDataStub> {
		Some(match self {
			Self::Text => MetastoreDataStub::Text,
			Self::Bytes => todo!(),
			Self::Integer { is_non_negative } => MetastoreDataStub::Integer {
				is_non_negative: *is_non_negative,
			},
			Self::Boolean => MetastoreDataStub::Boolean,
			Self::Float { is_non_negative } => MetastoreDataStub::Float {
				is_non_negative: *is_non_negative,
			},
			Self::Hash { hash_type } => MetastoreDataStub::Hash {
				hash_type: *hash_type,
			},
			Self::Reference { class } => MetastoreDataStub::Reference { class: *class },
		})
	}
}

// Get the [`CopperDataStub`] that is encoded into the given MetastoreDataStub.
// This should match `as_metastore_stub` above.
impl From<MetastoreDataStub> for CopperDataStub {
	fn from(value: MetastoreDataStub) -> Self {
		match value {
			MetastoreDataStub::Text => Self::Text,
			MetastoreDataStub::Binary => Self::Bytes,
			MetastoreDataStub::Blob => Self::Bytes,
			MetastoreDataStub::Integer { is_non_negative } => Self::Integer { is_non_negative },
			MetastoreDataStub::Boolean => Self::Boolean,
			MetastoreDataStub::Float { is_non_negative } => Self::Float { is_non_negative },
			MetastoreDataStub::Hash { hash_type } => Self::Hash { hash_type },
			MetastoreDataStub::Reference { class } => Self::Reference { class },
		}
	}
}

impl CopperDataStub {
	/// Iterate over all possible stubs
	pub fn iter_all() -> impl Iterator<Item = &'static Self> {
		[
			Self::Text,
			Self::Bytes,
			Self::Integer {
				is_non_negative: false,
			},
			Self::Integer {
				is_non_negative: true,
			},
			Self::Boolean,
			Self::Float {
				is_non_negative: true,
			},
			Self::Float {
				is_non_negative: false,
			},
			Self::Hash {
				hash_type: HashType::MD5,
			},
			Self::Hash {
				hash_type: HashType::SHA256,
			},
			Self::Hash {
				hash_type: HashType::SHA512,
			},
			//Self::Reference { class: ClassHandle },
		]
		.iter()
	}
}
