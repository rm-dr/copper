//#![warn(missing_docs)]

use std::{fmt::Debug, sync::Arc};

use ufo_util::mime::MimeType;

pub mod api;
//pub mod mem;
pub mod sea;

// TODO: error types
// TODO: rename
/// An immutable bit of data inside a pipeline.
/// These are instances of [`PipelineDataType`].
#[derive(Clone)]
pub enum StorageData {
	/// Typed, unset data
	None(StorageDataType),

	/// A block of text
	Text(Arc<String>),

	/// Binary data
	Binary {
		/// This data's media type
		format: MimeType,

		/// The data
		data: Arc<Vec<u8>>,
	},
}

impl Debug for StorageData {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::None(t) => write!(f, "None({:?})", t),
			Self::Text(s) => write!(f, "Text({})", s),
			Self::Binary { format, .. } => write!(f, "Binary({:?})", format),
		}
	}
}

impl StorageData {
	/// Transforms a data container into its type.
	pub fn get_type(&self) -> StorageDataType {
		match self {
			Self::None(t) => *t,
			Self::Text(_) => StorageDataType::Text,
			Self::Binary { .. } => StorageDataType::Binary,
		}
	}
}

/// A data type inside a pipeline.
/// Corresponds to [`PipelineData`]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum StorageDataType {
	/// Plain text
	Text,

	/// Binary data, in any format
	Binary,
}
