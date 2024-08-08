//! Datatypes and containers

use serde::Deserialize;
use smartstring::{LazyCompact, SmartString};
use std::{fmt::Debug, path::PathBuf, str::FromStr, sync::Arc};
use ufo_pipeline::api::{PipelineData, PipelineDataStub};
use ufo_storage::{
	api::{ClassHandle, ItemHandle},
	data::{StorageData, StorageDataType},
};
use ufo_util::mime::MimeType;

// TODO: rename
// TODO: remove all this and move to Storage

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashType {
	MD5,
	SHA256,
}

impl From<HashType> for ufo_storage::data::HashType {
	fn from(value: HashType) -> Self {
		match value {
			HashType::MD5 => ufo_storage::data::HashType::MD5,
			HashType::SHA256 => ufo_storage::data::HashType::SHA256,
		}
	}
}

impl From<ufo_storage::data::HashType> for HashType {
	fn from(value: ufo_storage::data::HashType) -> Self {
		match value {
			ufo_storage::data::HashType::MD5 => HashType::MD5,
			ufo_storage::data::HashType::SHA256 => HashType::SHA256,
		}
	}
}

/// Immutable bits of data.
/// These are instances of [`PipelineDataType`].
///
/// Cloning data should be very fast. Consider an [`Arc`]
/// if a variant holds lots of data.
///
/// Any variant that has a "deserialize" implementation
/// may be used as a parameter in certain nodes.
/// (for example, the `Constant` node's `value` field)
#[derive(Clone, Deserialize, Debug)]
#[serde(untagged)]
pub enum UFOData {
	/// Typed, unset data
	#[serde(skip)]
	None(UFODataStub),

	/// A block of text
	Text(Arc<String>),

	/// A filesystem path
	#[serde(skip)]
	Path(Arc<PathBuf>),

	/// An integer
	Integer(i128),

	/// A positive integer
	PositiveInteger(u128),

	/// A float
	Float(f64),

	/// A checksum
	#[serde(skip)]
	Hash {
		format: HashType,
		data: Arc<Vec<u8>>,
	},

	/// A reference to an item in a dataset
	#[serde(skip)]
	Reference {
		class: ClassHandle,
		item: ItemHandle,
	},

	/// Binary data
	#[serde(skip)]
	Binary {
		/// This data's media type
		format: MimeType,

		/// The data
		data: Arc<Vec<u8>>,
	},
}

impl PipelineData for UFOData {
	type DataStub = UFODataStub;

	fn as_stub(&self) -> Self::DataStub {
		match self {
			Self::None(t) => *t,
			Self::Text(_) => UFODataStub::Text,
			Self::Path(_) => UFODataStub::Path,
			Self::Integer(_) => UFODataStub::Integer,
			Self::PositiveInteger(_) => UFODataStub::PositiveInteger,
			Self::Float(_) => UFODataStub::Float,
			Self::Hash { format, .. } => UFODataStub::Hash { format: *format },
			Self::Binary { .. } => UFODataStub::Binary,
			Self::Reference { class, .. } => UFODataStub::Reference {
				class: class.clone(),
			},
		}
	}

	fn new_empty(stub: Self::DataStub) -> Self {
		Self::None(stub)
	}
}

impl UFOData {
	pub fn to_storage_data(&self) -> StorageData {
		match self {
			Self::None(t) => StorageData::None(t.to_storage_type()),
			Self::Text(t) => StorageData::Text((**t).clone()),
			Self::Path(p) => StorageData::Path(p.to_string_lossy().to_string()),
			Self::Integer(x) => StorageData::Integer(*x),
			Self::PositiveInteger(x) => StorageData::PositiveInteger(*x),
			Self::Float(x) => StorageData::Float(*x),
			Self::Hash { format, data } => StorageData::Hash {
				format: (*format).into(),
				data: (**data).clone(),
			},
			Self::Binary { format, data } => StorageData::Binary {
				format: format.clone(),
				data: (**data).clone(),
			},
			Self::Reference { class, item } => StorageData::Reference {
				class: *class,
				item: *item,
			},
		}
	}
}

/// A data type inside a pipeline.
/// Corresponds to [`PipelineData`]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum UFODataStub {
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
	Hash {
		format: HashType,
	},

	Reference {
		class: ClassHandle,
	},
}

impl PipelineDataStub for UFODataStub {}

// TODO: better error
impl FromStr for UFODataStub {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"text" => Ok(Self::Text),
			"binary" => Ok(Self::Binary),
			"path" => Ok(Self::Path),
			"reference" => todo!(),
			"hash::sha256" => Ok(Self::Hash {
				format: HashType::SHA256,
			}),
			_ => Err("bad data type".to_string()),
		}
	}
}

impl<'de> Deserialize<'de> for UFODataStub {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		let addr_str = SmartString::<LazyCompact>::deserialize(deserializer)?;
		let s = Self::from_str(&addr_str);
		s.map_err(serde::de::Error::custom)
	}
}

impl UFODataStub {
	pub fn to_storage_type(&self) -> StorageDataType {
		match self {
			Self::Binary => StorageDataType::Binary,
			Self::Text => StorageDataType::Text,
			Self::Integer => StorageDataType::Integer,
			Self::PositiveInteger => StorageDataType::PositiveInteger,
			Self::Float => StorageDataType::Float,
			Self::Path => StorageDataType::Path,
			Self::Hash { format } => StorageDataType::Hash {
				format: (*format).into(),
			},
			Self::Reference { class } => StorageDataType::Reference { class: *class },
		}
	}
}
