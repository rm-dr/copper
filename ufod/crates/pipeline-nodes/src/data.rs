use serde::{Deserialize, Serialize};
use std::{fmt::Debug, path::PathBuf, sync::Arc};
use ufo_ds_core::{
	data::{HashType, MetastoreData, MetastoreDataStub},
	handles::{ClassHandle, ItemIdx},
};
use ufo_pipeline::api::{PipelineData, PipelineDataStub};
use ufo_util::mime::MimeType;

/// Immutable bits of data inside a pipeline.
///
/// Cloning [`UFOData`] should be very fast. Consider wrapping
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
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UFOData {
	/// Typed, unset data
	#[serde(skip)]
	None(UFODataStub),

	/// A block of text
	Text(Arc<String>),

	/// An integer
	#[serde(skip)]
	Integer { value: i64, is_non_negative: bool },

	/// A boolean
	#[serde(skip)]
	Boolean(bool),

	/// A float
	#[serde(skip)]
	Float { value: f64, is_non_negative: bool },

	/// A checksum
	#[serde(skip)]
	Hash {
		format: HashType,
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

	#[serde(skip)]
	Reference {
		/// The item class this
		class: ClassHandle,

		/// The item
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

impl PipelineData for UFOData {
	type DataStubType = UFODataStub;

	fn as_stub(&self) -> Self::DataStubType {
		match self {
			Self::None(t) => *t,
			Self::Text(_) => UFODataStub::Text,
			Self::Integer {
				is_non_negative, ..
			} => UFODataStub::Integer {
				is_non_negative: *is_non_negative,
			},
			Self::Boolean(_) => UFODataStub::Boolean,
			Self::Float {
				is_non_negative, ..
			} => UFODataStub::Float {
				is_non_negative: *is_non_negative,
			},
			Self::Hash { format, .. } => UFODataStub::Hash { hash_type: *format },
			Self::Bytes { .. } => UFODataStub::Bytes,
			Self::Reference { class, .. } => UFODataStub::Reference { class: *class },
		}
	}

	fn new_empty(stub: Self::DataStubType) -> Self {
		Self::None(stub)
	}
}

impl UFOData {
	pub fn is_none(&self) -> bool {
		matches!(self, Self::None(_))
	}

	pub fn as_db_data(&self) -> Option<MetastoreData> {
		Some(match self {
			// These may not be converted to MetastoreData directly.
			// - Blobs must first be written to the blobstore
			// - Paths may not be stored in a metastore at all.
			UFOData::Bytes { .. } => return None,

			UFOData::Text(x) => MetastoreData::Text(x.clone()),
			UFOData::Boolean(x) => MetastoreData::Boolean(*x),

			UFOData::None(x) => {
				if let Some(stub) = x.as_metastore_stub() {
					MetastoreData::None(stub)
				} else {
					return None;
				}
			}
			UFOData::Hash { format, data } => MetastoreData::Hash {
				format: *format,
				data: data.clone(),
			},
			UFOData::Integer {
				value,
				is_non_negative,
			} => MetastoreData::Integer {
				value: *value,
				is_non_negative: *is_non_negative,
			},
			UFOData::Float {
				value,
				is_non_negative,
			} => MetastoreData::Float {
				value: *value,
				is_non_negative: *is_non_negative,
			},
			UFOData::Reference { class, item } => MetastoreData::Reference {
				class: *class,
				item: *item,
			},
		})
	}
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum UFODataStub {
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
	Reference { class: ClassHandle },
}

impl PipelineDataStub for UFODataStub {}

impl UFODataStub {
	/// Get the [`MetastoreDataStub`] that this [`UFODataStub`] encode to, if any.
	/// Not all [`UFODataStub`]s may be stored in a metastore.
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

// Get the UFODataStub that is encoded into the given MetastoreDataStub.
// This should match `as_metastore_stub` above.
impl From<MetastoreDataStub> for UFODataStub {
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

impl UFODataStub {
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
