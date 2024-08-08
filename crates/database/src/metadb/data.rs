use serde::Deserialize;
use smartstring::{LazyCompact, SmartString};
use std::{fmt::Debug, path::PathBuf, sync::Arc};
use ufo_pipeline::api::{PipelineData, PipelineDataStub};
use ufo_util::mime::MimeType;

use crate::blobstore::fs::store::FsBlobHandle;

use super::api::{ClassHandle, ItemHandle};

/// Bits of data inside a metadata db.
#[derive(Debug, Clone)]
pub enum MetaDbData {
	/// Typed, unset data
	None(MetaDbDataStub),

	/// A block of text
	Text(Arc<String>),

	/// A filesystem path
	Path(Arc<PathBuf>),

	/// An integer
	Integer(i64),

	/// A positive integer
	PositiveInteger(u64),

	/// A boolean
	Boolean(bool),

	/// A float
	Float(f64),

	/// A checksum
	Hash {
		format: HashType,
		data: Arc<Vec<u8>>,
	},

	/// Small binary data.
	/// This will be stored in the metadata db.
	Binary {
		/// This data's media type
		format: MimeType,

		/// The data
		data: Arc<Vec<u8>>,
	},

	/// Big binary data stored in the blob store.
	Blob { handle: FsBlobHandle },

	Reference {
		/// The item class this
		class: ClassHandle,

		/// The item
		item: ItemHandle,
	},
}

impl PipelineData for MetaDbData {
	type DataStub = MetaDbDataStub;

	fn as_stub(&self) -> Self::DataStub {
		match self {
			Self::None(t) => *t,
			Self::Text(_) => MetaDbDataStub::Text,
			Self::Path(_) => MetaDbDataStub::Path,
			Self::Integer(_) => MetaDbDataStub::Integer,
			Self::PositiveInteger(_) => MetaDbDataStub::PositiveInteger,
			Self::Boolean(_) => MetaDbDataStub::Boolean,
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

impl MetaDbData {
	pub fn is_none(&self) -> bool {
		matches!(self, Self::None(_))
	}

	pub fn is_blob(&self) -> bool {
		matches!(self, Self::Blob { .. })
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum HashType {
	MD5,
	SHA256,
	SHA512,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MetaDbDataStub {
	/// Plain text
	Text,

	/// Binary data, in any format
	Binary,

	/// Big binary data
	Blob,

	/// A filesystem path
	Path,

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

impl<'de> Deserialize<'de> for MetaDbDataStub {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		let addr_str = SmartString::<LazyCompact>::deserialize(deserializer)?;
		let s = Self::from_db_str(&addr_str);
		s.ok_or(serde::de::Error::custom(format!(
			"bad type string {}",
			addr_str
		)))
	}
}

impl PipelineDataStub for MetaDbDataStub {}

impl MetaDbDataStub {
	/// A string that represents this type in a database.
	pub fn to_db_str(&self) -> String {
		match self {
			Self::Text => "text".into(),
			Self::Binary => "binary".into(),
			Self::Blob => "blob".into(),
			Self::Boolean => "boolean".into(),
			Self::Path => "path".into(),
			Self::Integer => "integer".into(),
			Self::PositiveInteger => "positiveinteger".into(),
			Self::Float => "float".into(),
			Self::Hash { hash_type: format } => match format {
				HashType::MD5 => "hash::MD5".into(),
				HashType::SHA256 => "hash::SHA256".into(),
				HashType::SHA512 => "hash::SHA512".into(),
			},
			Self::Reference { class } => format!("reference::{}", u32::from(*class)),
		}
	}

	/// A string that represents this type in a database.
	pub fn from_db_str(s: &str) -> Option<Self> {
		// Static strings
		let q = match s {
			"text" => Some(Self::Text),
			"binary" => Some(Self::Binary),
			"path" => Some(Self::Path),
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
			return q;
		}

		if let Some(c) = s.strip_prefix("reference::") {
			let n: u32 = c.parse().ok()?;
			return Some(Self::Reference { class: n.into() });
		}

		return None;
	}
}
