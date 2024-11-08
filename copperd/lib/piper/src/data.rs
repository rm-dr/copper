use copper_itemdb::{AttrData, AttrDataStub};
use copper_util::{HashType, MimeType};
use smartstring::{LazyCompact, SmartString};
use std::{fmt::Debug, sync::Arc};

use crate::base::PipelineData;

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
#[derive(Debug, Clone)]
pub enum PipeData {
	/// A block of text
	Text { value: SmartString<LazyCompact> },

	/// An integer
	Integer { value: i64, is_non_negative: bool },

	/// A boolean
	Boolean { value: bool },

	/// A float
	Float { value: f64, is_non_negative: bool },

	/// A checksum
	Hash { hash_type: HashType, data: Vec<u8> },

	/// Arbitrary binary data.
	/// This will be stored in the metadata db.
	Blob {
		/// The data
		source: BytesSource,
	},

	TransactionActionResult {
		action_idx: usize,
		result_type: AttrDataStub,
	},
}

#[derive(Debug, Clone)]
pub enum BytesSource {
	Array {
		mime: MimeType,
		data: Arc<Vec<u8>>,
	},
	Stream {
		mime: MimeType,
		receiver: async_broadcast::Receiver<Arc<Vec<u8>>>,
	},
	S3 {
		key: SmartString<LazyCompact>,
		bucket: SmartString<LazyCompact>,
	},
}

impl TryFrom<AttrData> for PipeData {
	type Error = ();

	fn try_from(value: AttrData) -> Result<Self, Self::Error> {
		return Ok(match value {
			AttrData::Blob { .. } => return Err(()),
			AttrData::Reference { .. } => return Err(()),

			AttrData::Text { value } => Self::Text { value },
			AttrData::Boolean { value } => Self::Boolean { value },
			AttrData::Hash { hash_type, data } => Self::Hash { hash_type, data },

			AttrData::Float {
				value,
				is_non_negative,
			} => Self::Float {
				value,
				is_non_negative,
			},

			AttrData::Integer {
				value,
				is_non_negative,
			} => Self::Integer {
				value,
				is_non_negative,
			},
		});
	}
}

impl TryInto<AttrData> for PipeData {
	type Error = ();

	fn try_into(self) -> Result<AttrData, Self::Error> {
		return Ok(match self {
			Self::Blob { .. } => return Err(()),
			Self::TransactionActionResult { .. } => return Err(()),

			Self::Text { value } => AttrData::Text { value },
			Self::Boolean { value } => AttrData::Boolean { value },
			Self::Hash { hash_type, data } => AttrData::Hash { hash_type, data },

			Self::Float {
				value,
				is_non_negative,
			} => AttrData::Float {
				value,
				is_non_negative,
			},

			Self::Integer {
				value,
				is_non_negative,
			} => AttrData::Integer {
				value,
				is_non_negative,
			},
		});
	}
}

impl PipelineData for PipeData {}
