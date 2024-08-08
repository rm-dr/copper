//! Datatypes and containers

use serde::Deserialize;
use smartstring::{LazyCompact, SmartString};
use std::{
	fmt::{Debug, Display},
	str::FromStr,
	sync::Arc,
};
use ufo_pipeline::api::{PipelineData, PipelineDataStub};
use ufo_storage::{
	api::{ClassHandle, ItemHandle},
	StorageData, StorageDataType,
};
use ufo_util::mime::MimeType;

// TODO: no clone vec

// TODO: rename
/// An immutable bit of data inside a pipeline.
/// These are instances of [`PipelineDataType`].
///
/// Any variant that has a "deserialize" implementation
/// may be used as a parameter in certain nodes.
/// (for example, the `Constant` node's `value` field)
#[derive(Clone, Deserialize)]
#[serde(untagged)]
pub enum UFOData {
	/// Typed, unset data
	#[serde(skip)]
	None(UFODataStub),

	/// A block of text
	Text(Arc<String>),

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

impl Debug for UFOData {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::None(t) => write!(f, "None({})", t),
			Self::Text(s) => write!(f, "Text({})", s),
			Self::Binary { format, .. } => write!(f, "Binary({:?})", format),
			Self::Reference { class, item } => write!(f, "Reference({class:?} {item:?})"),
		}
	}
}

impl PipelineData for UFOData {
	type DataStub = UFODataStub;

	fn as_stub(&self) -> Self::DataStub {
		match self {
			Self::None(t) => *t,
			Self::Text(_) => UFODataStub::Text,
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
			Self::Text(t) => StorageData::Text(t.clone()),
			Self::Binary { format, data } => StorageData::Binary {
				format: format.clone(),
				data: data.clone(),
			},
			Self::Reference { class, item } => StorageData::Reference {
				class: class.clone(),
				item: item.clone(),
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

	Reference {
		class: ClassHandle,
	},
}

impl PipelineDataStub for UFODataStub {}

impl Display for UFODataStub {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Text => write!(f, "Text"),
			Self::Binary => write!(f, "Binary"),
			Self::Reference { class } => write!(f, "Reference({class:?})"),
		}
	}
}

// TODO: better error
impl FromStr for UFODataStub {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"text" => Ok(Self::Text),
			"binary" => Ok(Self::Binary),
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
			Self::Reference { class } => StorageDataType::Reference {
				class: class.clone(),
			},
		}
	}
}
