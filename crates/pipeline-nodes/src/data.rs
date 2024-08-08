use serde::Deserialize;
use std::{fmt::Debug, path::PathBuf, sync::Arc};
use ufo_database::metastore::{
	data::{HashType, MetastoreData, MetastoreDataStub},
	handles::{ClassHandle, ItemHandle},
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
	None(MetastoreDataStub),

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

	/// A boolean
	#[serde(skip)]
	Boolean(bool),

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
		fragment: Arc<Vec<u8>>,

		/// Is this the last fragment?
		is_last: bool,
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
	type DataStub = MetastoreDataStub;

	fn as_stub(&self) -> Self::DataStub {
		match self {
			Self::None(t) => *t,
			Self::Text(_) => MetastoreDataStub::Text,
			Self::Path(_) => MetastoreDataStub::Path,
			Self::Integer(_) => MetastoreDataStub::Integer,
			Self::PositiveInteger(_) => MetastoreDataStub::PositiveInteger,
			Self::Boolean(_) => MetastoreDataStub::Boolean,
			Self::Float(_) => MetastoreDataStub::Float,
			Self::Hash { format, .. } => MetastoreDataStub::Hash { hash_type: *format },
			Self::Binary { .. } => MetastoreDataStub::Binary,
			Self::Blob { .. } => MetastoreDataStub::Blob,
			Self::Reference { class, .. } => MetastoreDataStub::Reference { class: *class },
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

	pub fn as_db_data(&self) -> Option<MetastoreData> {
		Some(match self {
			UFOData::Blob { .. } => return None,

			UFOData::None(x) => MetastoreData::None(*x),
			UFOData::Text(x) => MetastoreData::Text(x.clone()),
			UFOData::Float(x) => MetastoreData::Float(*x),
			UFOData::Path(x) => MetastoreData::Path(x.clone()),
			UFOData::Boolean(x) => MetastoreData::Boolean(*x),
			UFOData::Hash { format, data } => MetastoreData::Hash {
				format: *format,
				data: data.clone(),
			},
			UFOData::Binary { format, data } => MetastoreData::Binary {
				format: format.clone(),
				data: data.clone(),
			},
			UFOData::Integer(x) => MetastoreData::Integer(*x),
			UFOData::PositiveInteger(x) => MetastoreData::PositiveInteger(*x),
			UFOData::Reference { class, item } => MetastoreData::Reference {
				class: *class,
				item: *item,
			},
		})
	}
}
