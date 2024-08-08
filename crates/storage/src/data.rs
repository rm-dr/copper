use std::fmt::Debug;
use ufo_util::mime::MimeType;

use crate::api::{ClassHandle, ItemHandle};

#[derive(Debug, Clone)]
pub enum StorageData {
	/// Typed, unset data
	None(StorageDataType),

	/// A block of text
	Text(String),

	/// A filesystem path
	Path(String),

	/// An integer
	Integer(i128),

	/// A positive integer
	PositiveInteger(u128),

	/// A float
	Float(f64),

	/// A checksum
	Hash { format: HashType, data: Vec<u8> },

	/// Binary data
	Binary {
		/// This data's media type
		format: MimeType,

		/// The data
		data: Vec<u8>,
	},

	Reference {
		class: ClassHandle,
		item: ItemHandle,
	},
}

impl StorageData {
	/// Transforms a data container into its type.
	pub fn get_type(&self) -> StorageDataType {
		match self {
			Self::None(t) => *t,
			Self::Text(_) => StorageDataType::Text,
			Self::Binary { .. } => StorageDataType::Binary,
			Self::Path(_) => StorageDataType::Path,
			Self::Integer(_) => StorageDataType::Integer,
			Self::PositiveInteger(_) => StorageDataType::PositiveInteger,
			Self::Float(_) => StorageDataType::Float,
			Self::Hash { format, .. } => StorageDataType::Hash { format: *format },
			Self::Reference { class, .. } => StorageDataType::Reference {
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
pub enum StorageDataType {
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

impl StorageDataType {
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
