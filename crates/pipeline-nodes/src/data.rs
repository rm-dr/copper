use async_broadcast::Receiver;
use serde::Deserialize;
use std::{fmt::Debug, path::PathBuf, sync::Arc};
use ufo_metadb::{
	api::{ClassHandle, ItemHandle},
	data::{HashType, MetaDbData, MetaDbDataStub},
};
use ufo_pipeline::api::PipelineData;
use ufo_util::mime::MimeType;

/// Immutable bits of data inside a pipeline.
///
/// Cloning [`UFOData`] should be very fast. Consider wrapping
/// big containers in an [`Arc`].
///
/// Any variant that has a "deserialize" implementation
/// may be used as a parameter in certain nodes.
/// (for example, the `Constant` node's `value` field)
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum UFOData {
	/// Typed, unset data
	#[serde(skip)]
	None(MetaDbDataStub),

	/// A block of text
	Text(Arc<String>),

	/// A filesystem path
	#[serde(skip)]
	Path(Arc<PathBuf>),

	/// An integer
	#[serde(skip)]
	Integer(i64),

	/// A positive integer
	#[serde(skip)]
	PositiveInteger(u64),

	/// A float
	#[serde(skip)]
	Float(f64),

	/// A checksum
	#[serde(skip)]
	Hash {
		format: HashType,
		data: Arc<Vec<u8>>,
	},

	/// Small binary data.
	/// This will be stored in the metadata db.
	#[serde(skip)]
	Binary {
		/// This data's media type
		format: MimeType,

		/// The data
		data: Arc<Vec<u8>>,
	},

	/// Big binary data.
	/// This will be stored in the blob store.
	#[serde(skip)]
	Blob {
		/// This data's media type
		format: MimeType,

		/// A receiver that provides data
		data: Receiver<Arc<Vec<u8>>>,
	},

	#[serde(skip)]
	Reference {
		/// The item class this
		class: ClassHandle,

		/// The item
		item: ItemHandle,
	},
}

// TODO: better debug

impl PipelineData for UFOData {
	type DataStub = MetaDbDataStub;

	fn as_stub(&self) -> Self::DataStub {
		match self {
			Self::None(t) => *t,
			Self::Text(_) => MetaDbDataStub::Text,
			Self::Path(_) => MetaDbDataStub::Path,
			Self::Integer(_) => MetaDbDataStub::Integer,
			Self::PositiveInteger(_) => MetaDbDataStub::PositiveInteger,
			Self::Float(_) => MetaDbDataStub::Float,
			Self::Hash { format, .. } => MetaDbDataStub::Hash { hash_type: *format },
			Self::Binary { .. } => MetaDbDataStub::Binary,
			Self::Blob { .. } => MetaDbDataStub::Blob,
			Self::Reference { class, .. } => MetaDbDataStub::Reference { class: *class },
		}
	}

	fn new_empty(stub: Self::DataStub) -> Self {
		Self::None(stub)
	}
}

impl UFOData {
	pub fn is_none(&self) -> bool {
		matches!(self, Self::None(_))
	}

	pub fn is_blob(&self) -> bool {
		matches!(self, Self::Blob { .. })
	}

	pub fn as_db_data(&self) -> Option<MetaDbData> {
		Some(match self {
			UFOData::Blob { .. } => return None,

			UFOData::None(x) => MetaDbData::None(*x),
			UFOData::Text(x) => MetaDbData::Text(x.clone()),
			UFOData::Float(x) => MetaDbData::Float(*x),
			UFOData::Path(x) => MetaDbData::Path(x.clone()),
			UFOData::Hash { format, data } => MetaDbData::Hash {
				format: *format,
				data: data.clone(),
			},
			UFOData::Binary { format, data } => MetaDbData::Binary {
				format: format.clone(),
				data: data.clone(),
			},
			UFOData::Integer(x) => MetaDbData::Integer(*x),
			UFOData::PositiveInteger(x) => MetaDbData::PositiveInteger(*x),
			UFOData::Reference { class, item } => MetaDbData::Reference {
				class: *class,
				item: *item,
			},
		})
	}
}
