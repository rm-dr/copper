use serde::Deserialize;
use smartstring::{LazyCompact, SmartString};
use std::{fmt::Debug, path::PathBuf, str::FromStr, sync::Arc};
use ufo_pipeline::api::{PipelineData, PipelineDataStub};
use ufo_util::mime::MimeType;

use crate::api::{ClassHandle, ItemHandle};

/// Immutable bits of data.
///
/// Cloning [`StorageData`] should be very fast. Consider wrapping
/// big containers in an [`Arc`].
///
/// TODO: split deserialize?
/// Any variant that has a "deserialize" implementation
/// may be used as a parameter in certain nodes.
/// (for example, the `Constant` node's `value` field)
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum StorageData {
	/// Typed, unset data
	#[serde(skip)]
	None(StorageDataStub),

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

	/// Binary data
	#[serde(skip)]
	Binary {
		/// This data's media type
		format: MimeType,

		/// The data
		data: Arc<Vec<u8>>,
	},

	#[serde(skip)]
	Reference {
		/// The item class this
		class: ClassHandle,

		/// The item
		item: ItemHandle,
	},
}

impl PipelineData for StorageData {
	type DataStub = StorageDataStub;

	fn as_stub(&self) -> Self::DataStub {
		match self {
			Self::None(t) => *t,
			Self::Text(_) => StorageDataStub::Text,
			Self::Path(_) => StorageDataStub::Path,
			Self::Integer(_) => StorageDataStub::Integer,
			Self::PositiveInteger(_) => StorageDataStub::PositiveInteger,
			Self::Float(_) => StorageDataStub::Float,
			Self::Hash { format, .. } => StorageDataStub::Hash { format: *format },
			Self::Binary { .. } => StorageDataStub::Binary,
			Self::Reference { class, .. } => StorageDataStub::Reference {
				class: class.clone(),
			},
		}
	}

	fn new_empty(stub: Self::DataStub) -> Self {
		Self::None(stub)
	}
}

impl StorageData {
	/// Transforms a data container into its type.
	pub fn get_type(&self) -> StorageDataStub {
		match self {
			Self::None(t) => *t,
			Self::Text(_) => StorageDataStub::Text,
			Self::Binary { .. } => StorageDataStub::Binary,
			Self::Path(_) => StorageDataStub::Path,
			Self::Integer(_) => StorageDataStub::Integer,
			Self::PositiveInteger(_) => StorageDataStub::PositiveInteger,
			Self::Float(_) => StorageDataStub::Float,
			Self::Hash { format, .. } => StorageDataStub::Hash { format: *format },
			Self::Reference { class, .. } => StorageDataStub::Reference {
				class: class.clone(),
			},
		}
	}

	/// `true` if this is `Self::None`
	pub fn is_none(&self) -> bool {
		matches!(self, Self::None(_))
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashType {
	MD5,
	SHA256,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum StorageDataStub {
	/// Plain text
	Text,

	/// Binary data, in any format
	Binary,

	/// A filesystem path
	Path,

	/// An integer
	Integer,

	/// A positive integer
	PositiveInteger,

	/// A float
	Float,

	/// A checksum
	Hash { format: HashType },

	/// A reference to an item
	Reference { class: ClassHandle },
}

// TODO: better error
impl FromStr for StorageDataStub {
	type Err = &'static str;

	fn from_str(s: &str) -> Result<Self, &'static str> {
		match s {
			"text" => Ok(Self::Text),
			"binary" => Ok(Self::Binary),
			"path" => Ok(Self::Path),
			"reference" => todo!(),
			"hash::sha256" => Ok(Self::Hash {
				format: HashType::SHA256,
			}),
			_ => Err("bad data type"),
		}
	}
}

impl<'de> Deserialize<'de> for StorageDataStub {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		let addr_str = SmartString::<LazyCompact>::deserialize(deserializer)?;
		let s = Self::from_str(&addr_str);
		s.map_err(serde::de::Error::custom)
	}
}

impl PipelineDataStub for StorageDataStub {}

impl StorageDataStub {
	/// A string that represents this type in a database.
	pub fn to_db_str(&self) -> String {
		match self {
			Self::Text => "text".into(),
			Self::Binary => "binary".into(),
			Self::Path => "path".into(),
			Self::Integer => "integer".into(),
			Self::PositiveInteger => "postiveinteger".into(),
			Self::Float => "float".into(),
			Self::Hash { format } => match format {
				HashType::MD5 => "hash::md5".into(),
				HashType::SHA256 => "hash::sha256".into(),
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
			"integer" => Some(Self::Integer),
			"positiveinteger" => Some(Self::PositiveInteger),
			"float" => Some(Self::Float),
			"hash::md5" => Some(Self::Hash {
				format: HashType::MD5,
			}),
			"hash::sha256" => Some(Self::Hash {
				format: HashType::SHA256,
			}),
			_ => None,
		};

		if q.is_some() {
			return q;
		}

		if s.starts_with("reference::") {
			let n: Option<u32> = s[11..].parse().ok();
			if n.is_none() {
				return None;
			}
			return Some(Self::Reference {
				class: n.unwrap().into(),
			});
		}

		return None;
	}
}
