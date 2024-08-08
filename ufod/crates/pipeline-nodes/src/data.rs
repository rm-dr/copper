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
	Integer(i64),

	/// A filesystem path.
	/// This cannot be stored inside a metastore.
	#[serde(skip)]
	Path(PathBuf),

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
		mime: MimeType,

		/// The data
		data: Arc<Vec<u8>>,
	},

	/// Big binary data.
	/// This will be stored in the blob store.
	#[serde(skip)]
	Blob {
		/// This data's media type
		mime: MimeType,

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
		item: ItemIdx,
	},
}

impl PipelineData for UFOData {
	type DataStubType = UFODataStub;

	fn as_stub(&self) -> Self::DataStubType {
		match self {
			Self::None(t) => *t,
			Self::Text(_) => UFODataStub::Text,
			Self::Path(_) => UFODataStub::Path,
			Self::Integer(_) => UFODataStub::Integer,
			Self::PositiveInteger(_) => UFODataStub::PositiveInteger,
			Self::Boolean(_) => UFODataStub::Boolean,
			Self::Float(_) => UFODataStub::Float,
			Self::Hash { format, .. } => UFODataStub::Hash { hash_type: *format },
			Self::Binary { .. } => UFODataStub::Binary,
			Self::Blob { .. } => UFODataStub::Blob,
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

	pub fn is_blob(&self) -> bool {
		matches!(self, Self::Blob { .. })
	}

	pub fn as_db_data(&self) -> Option<MetastoreData> {
		Some(match self {
			// These may not be converted to MetastoreData directly.
			// - Blobs must first be written to the blobstore
			// - Paths may not be stored in a metastore at all.
			UFOData::Blob { .. } => return None,
			UFOData::Path(_) => return None,

			UFOData::None(x) => {
				if let Some(stub) = x.as_metastore_stub() {
					MetastoreData::None(stub)
				} else {
					return None;
				}
			}
			UFOData::Text(x) => MetastoreData::Text(x.clone()),
			UFOData::Float(x) => MetastoreData::Float(*x),
			UFOData::Boolean(x) => MetastoreData::Boolean(*x),
			UFOData::Hash { format, data } => MetastoreData::Hash {
				format: *format,
				data: data.clone(),
			},
			UFOData::Binary { mime: format, data } => MetastoreData::Binary {
				mime: format.clone(),
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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum UFODataStub {
	/// Plain text
	Text,

	/// A filesystem path
	Path,

	/// Small binary data, in any format
	Binary,

	/// Big binary data
	Blob,

	/// An integer
	Integer,

	/// A positive integer
	PositiveInteger,

	/// A boolean
	Boolean,

	/// A float
	Float,

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
			Self::Path => return None,

			Self::Text => MetastoreDataStub::Text,
			Self::Binary => MetastoreDataStub::Binary,
			Self::Blob => MetastoreDataStub::Blob,
			Self::Integer => MetastoreDataStub::Integer,
			Self::PositiveInteger => MetastoreDataStub::PositiveInteger,
			Self::Boolean => MetastoreDataStub::Boolean,
			Self::Float => MetastoreDataStub::Float,
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
			MetastoreDataStub::Binary => Self::Binary,
			MetastoreDataStub::Blob => Self::Blob,
			MetastoreDataStub::Integer => Self::Integer,
			MetastoreDataStub::PositiveInteger => Self::PositiveInteger,
			MetastoreDataStub::Boolean => Self::Boolean,
			MetastoreDataStub::Float => Self::Float,
			MetastoreDataStub::Hash { hash_type } => Self::Hash { hash_type },
			MetastoreDataStub::Reference { class } => Self::Reference { class },
		}
	}
}

impl<'de> Deserialize<'de> for UFODataStub {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		// Make sure this matches `Serialize`!
		let s = String::deserialize(deserializer)?;

		let stub = 'block: {
			// Static strings
			let q = match &s[..] {
				"text" => Some(Self::Text),
				"binary" => Some(Self::Binary),
				"blob" => Some(Self::Blob),
				"boolan" => Some(Self::Boolean),
				"integer" => Some(Self::Integer),
				"positiveinteger" => Some(Self::PositiveInteger),
				"float" => Some(Self::Float),
				"hash::MD5" => Some(Self::Hash {
					hash_type: HashType::MD5,
				}),
				"hash::SHA256" => Some(Self::Hash {
					hash_type: HashType::SHA256,
				}),
				"hash::SHA512" => Some(Self::Hash {
					hash_type: HashType::SHA512,
				}),
				_ => None,
			};

			if q.is_some() {
				break 'block q;
			}

			if let Some(c) = s.strip_prefix("reference::") {
				let n: u32 = if let Ok(n) = c.parse() {
					n
				} else {
					break 'block None;
				};
				break 'block Some(Self::Reference { class: n.into() });
			}

			None
		};

		stub.ok_or(serde::de::Error::custom(format!("bad type string {}", s)))
	}
}

impl Serialize for UFODataStub {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		// Make sure this matches `Deserialize`!
		let s = match self {
			Self::Text => "text".into(),
			Self::Path => "path".into(),
			Self::Binary => "binary".into(),
			Self::Blob => "blob".into(),
			Self::Boolean => "boolean".into(),
			Self::Integer => "integer".into(),
			Self::PositiveInteger => "positiveinteger".into(),
			Self::Float => "float".into(),
			Self::Hash { hash_type: format } => match format {
				HashType::MD5 => "hash::MD5".into(),
				HashType::SHA256 => "hash::SHA256".into(),
				HashType::SHA512 => "hash::SHA512".into(),
			},
			Self::Reference { class } => format!("reference::{}", u32::from(*class)),
		};
		s.serialize(serializer)
	}
}

impl UFODataStub {
	/// Iterate over all possible stubs
	pub fn iter_all() -> impl Iterator<Item = &'static Self> {
		[
			Self::Text,
			Self::Binary,
			Self::Blob,
			Self::Path,
			Self::Integer,
			Self::PositiveInteger,
			Self::Boolean,
			Self::Float,
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
