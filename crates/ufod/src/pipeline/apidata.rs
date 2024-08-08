//! Structs that represent raw data

use serde::{Deserialize, Serialize};
use smartstring::{LazyCompact, SmartString};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema, Debug)]
pub enum ApiDataStub {
	Text,
	Blob,
	Integer,
	PositiveInteger,
	Boolean,
	Float,
}

/// Raw data that can be uploaded through the api
#[derive(Serialize, Deserialize, ToSchema, Debug)]
pub enum ApiData {
	/// Typed, unset data
	None(ApiDataStub),

	/// A block of text
	Text(String),

	/// A large file we've previously uploaded.
	/// TODO: this can become a Blob, a Path, or a Binary.
	Blob {
		#[schema(value_type = String)]
		file_name: SmartString<LazyCompact>,
	},

	/// An integer
	Integer(i64),

	/// A positive integer
	PositiveInteger(u64),

	/// A boolean
	Boolean(bool),

	/// A float
	Float(f64),
	/*
	/// A checksum
	#[serde(skip)]
	Hash {
		format: HashType,
		data: Arc<Vec<u8>>,
	},

	#[serde(skip)]
	Reference {
		/// The item class this
		class: ClassHandle,

		/// The item
		item: ItemHandle,
	},
	*/
}