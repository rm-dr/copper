use copper_storaged::{AttrData, AttrDataStub, ClassId, ItemId};
use copper_util::{mime::MimeType, HashType};
use serde::{Deserialize, Serialize};
use smartstring::{LazyCompact, SmartString};
use std::{fmt::Debug, sync::Arc};
use url::Url;
use utoipa::ToSchema;

use crate::base::{PipelineData, PipelineDataStub};

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
#[derive(Deserialize, Debug, Clone, ToSchema)]
#[serde(tag = "type")]
pub enum PipeData {
	/// Typed, unset data
	None { data_type: AttrDataStub },

	/// A block of text
	Text { value: SmartString<LazyCompact> },

	/// An integer
	Integer { value: i64, is_non_negative: bool },

	/// A boolean
	Boolean { value: bool },

	/// A float
	Float { value: f64, is_non_negative: bool },

	/// A checksum
	#[serde(skip)]
	Hash { hash_type: HashType, data: Vec<u8> },

	/// Arbitrary binary data.
	/// This will be stored in the metadata db.
	#[serde(skip)]
	Blob {
		/// This data's media type
		mime: MimeType,

		/// The data
		source: BytesSource,
	},

	#[serde(skip)]
	Reference {
		/// The item's class
		class: ClassId,

		/// The item
		item: ItemId,
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

impl PipelineData for PipeData {
	type DataStubType = PipeDataStub;

	fn as_stub(&self) -> Self::DataStubType {
		match self {
			Self::None { data_type } => PipeDataStub::Plain {
				data_type: *data_type,
			},

			Self::Text { .. } => PipeDataStub::Plain {
				data_type: AttrDataStub::Text,
			},

			Self::Integer {
				is_non_negative, ..
			} => PipeDataStub::Plain {
				data_type: AttrDataStub::Integer {
					is_non_negative: *is_non_negative,
				},
			},

			Self::Boolean { .. } => PipeDataStub::Plain {
				data_type: AttrDataStub::Boolean,
			},

			Self::Float {
				is_non_negative, ..
			} => PipeDataStub::Plain {
				data_type: AttrDataStub::Float {
					is_non_negative: *is_non_negative,
				},
			},

			Self::Hash {
				hash_type: format, ..
			} => PipeDataStub::Plain {
				data_type: AttrDataStub::Hash { hash_type: *format },
			},

			Self::Blob { .. } => PipeDataStub::Plain {
				data_type: AttrDataStub::Blob,
			},

			Self::Reference { class, .. } => PipeDataStub::Plain {
				data_type: AttrDataStub::Reference { class: *class },
			},
		}
	}
}

impl PipeData {
	pub fn is_none(&self) -> bool {
		matches!(self, Self::None { .. })
	}
}

impl TryFrom<AttrData> for PipeData {
	type Error = ();

	fn try_from(value: AttrData) -> Result<Self, Self::Error> {
		return Ok(match value {
			AttrData::Blob { .. } => return Err(()),
			AttrData::Text { value } => Self::Text { value },
			AttrData::Boolean { value } => Self::Boolean { value },
			AttrData::Hash { hash_type, data } => Self::Hash { hash_type, data },

			AttrData::None { data_type } => Self::None {
				data_type: data_type.into(),
			},

			AttrData::Reference { class, item } => Self::Reference {
				class: class.into(),
				item: item.into(),
			},

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
			Self::Text { value } => AttrData::Text { value },
			Self::Boolean { value } => AttrData::Boolean { value },
			Self::Hash { hash_type, data } => AttrData::Hash { hash_type, data },

			Self::None { data_type } => AttrData::None {
				data_type: data_type,
			},

			Self::Reference { class, item } => AttrData::Reference {
				class: class.into(),
				item: item.into(),
			},

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

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type")]
pub enum PipeDataStub {
	Plain { data_type: AttrDataStub },
}

impl PipelineDataStub for PipeDataStub {
	fn is_subset_of(&self, superset: &Self) -> bool {
		if self == superset {
			return true;
		}

		return false;
	}
}
