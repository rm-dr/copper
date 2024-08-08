//! Datatypes and containers

use serde::Deserialize;
use smartstring::{LazyCompact, SmartString};
use std::{
	fmt::{Debug, Display},
	str::FromStr,
	sync::Arc,
};
use ufo_storage::{
	api::{ClassHandle, ItemHandle},
	StorageData, StorageDataType,
};
use ufo_util::mime::MimeType;

// TODO: no clone vec

// TODO: rename
/// An immutable bit of data inside a pipeline.
/// These are instances of [`PipelineDataType`].
#[derive(Clone)]
pub enum PipelineData {
	/// Typed, unset data
	None(PipelineDataType),

	/// A block of text
	Text(Arc<String>),

	Reference {
		class: ClassHandle,
		item: ItemHandle,
	},

	/// Binary data
	Binary {
		/// This data's media type
		format: MimeType,

		/// The data
		data: Arc<Vec<u8>>,
	},
}

impl Debug for PipelineData {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::None(t) => write!(f, "None({})", t),
			Self::Text(s) => write!(f, "Text({})", s),
			Self::Binary { format, .. } => write!(f, "Binary({:?})", format),
			Self::Reference { class, item } => write!(f, "Reference({class:?} {item:?})"),
		}
	}
}

impl PipelineData {
	/// Transforms a data container into its type.
	pub fn get_type(&self) -> PipelineDataType {
		match self {
			Self::None(t) => *t,
			Self::Text(_) => PipelineDataType::Text,
			Self::Binary { .. } => PipelineDataType::Binary,
			Self::Reference { class, .. } => PipelineDataType::Reference {
				class: class.clone(),
			},
		}
	}

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
pub enum PipelineDataType {
	/// Plain text
	Text,

	/// Binary data, in any format
	Binary,

	Reference {
		class: ClassHandle,
	},
}

impl Display for PipelineDataType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Text => write!(f, "Text"),
			Self::Binary => write!(f, "Binary"),
			Self::Reference { class } => write!(f, "Reference({class:?})"),
		}
	}
}

// TODO: better error
impl FromStr for PipelineDataType {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"text" => Ok(Self::Text),
			"binary" => Ok(Self::Binary),
			_ => Err("bad data type".to_string()),
		}
	}
}

impl<'de> Deserialize<'de> for PipelineDataType {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		let addr_str = SmartString::<LazyCompact>::deserialize(deserializer)?;
		let s = Self::from_str(&addr_str);
		s.map_err(serde::de::Error::custom)
	}
}

impl PipelineDataType {
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
