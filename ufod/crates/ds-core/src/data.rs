use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, sync::Arc};
use ufo_util::mime::MimeType;
use utoipa::ToSchema;

use super::handles::{ClassHandle, ItemIdx};
use crate::api::blob::BlobHandle;

/// Bits of data inside a metadata db.
#[derive(Debug, Clone)]
pub enum MetastoreData {
	/// Typed, unset data
	None(MetastoreDataStub),

	/// A block of text
	Text(Arc<String>),

	/// An integer
	Integer { value: i64, is_non_negative: bool },

	/// A float
	Float { value: f64, is_non_negative: bool },

	/// A boolean
	Boolean(bool),

	/// A checksum
	Hash {
		format: HashType,
		data: Arc<Vec<u8>>,
	},

	/// Small binary data.
	/// This will be stored in the metadata db.
	Binary {
		/// This data's media type
		mime: MimeType,

		/// The data
		data: Arc<Vec<u8>>,
	},

	/// Big binary data stored in the blob store.
	Blob { handle: BlobHandle },

	Reference {
		/// The item class this reference points to
		class: ClassHandle,

		/// The item
		item: ItemIdx,
	},
}

impl MetastoreData {
	pub fn is_none(&self) -> bool {
		matches!(self, Self::None(_))
	}

	pub fn is_blob(&self) -> bool {
		matches!(self, Self::Blob { .. })
	}

	/// Convert a hash to a hex string
	pub fn hash_to_string(data: &Vec<u8>) -> String {
		data.iter().map(|x| format!("{:02X}", x)).join("")
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, ToSchema)]
pub enum HashType {
	MD5,
	SHA256,
	SHA512,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type")]
pub enum MetastoreDataStub {
	/// Plain text
	Text,

	/// Binary data, in any format
	Binary,

	/// Big binary data
	Blob,

	/// An integer
	Integer { is_non_negative: bool },

	/// A float
	Float { is_non_negative: bool },

	/// A boolean
	Boolean,

	/// A checksum
	Hash { hash_type: HashType },

	/// A reference to an item
	Reference {
		#[schema(value_type = u32)]
		class: ClassHandle,
	},
}
