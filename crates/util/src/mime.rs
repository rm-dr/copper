//! Media type utilities

use serde::{Deserialize, Serialize};
use smartstring::{LazyCompact, SmartString};
use std::fmt::Display;

/// A media type, conveniently parsed
#[allow(missing_docs)]
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum MimeType {
	/// A mimetype we didn't recognize
	Unknown(SmartString<LazyCompact>),

	/// An unstructured binary blob
	Blob,

	// Images
	Png,
	Jpg,

	// Audo
	Flac,
	Mp3,
}

impl From<&str> for MimeType {
	// Must match `to_str` below, but may provide other alternatives.
	fn from(value: &str) -> Self {
		match value {
			"application/octet-stream" => Self::Blob,
			"image/png" => Self::Png,
			"image/jpg" => Self::Jpg,
			"image/jpeg" => Self::Jpg,
			_ => Self::Unknown(value.into()),
		}
	}
}

impl From<String> for MimeType {
	fn from(value: String) -> Self {
		Self::from(&value[..])
	}
}

impl MimeType {
	// Must match `From<String>` above

	/// Get a string representation of this mimetype.
	///
	/// The following always holds
	/// ```no_run
	/// // x: MimeType
	/// MimeType::from(x.to_str()) == x
	/// ```
	///
	/// The following might not hold:
	/// ```no_run
	/// // y: &str
	/// MimeType::from(y).to_str() == y
	/// ```
	pub fn to_db_str(&self) -> &str {
		match self {
			Self::Blob => "application/octet-stream",
			Self::Png => "image/png",
			Self::Jpg => "image/jpeg",
			Self::Flac => "audio/flac",
			Self::Mp3 => "audio/mp3",
			Self::Unknown(x) => x,
		}
	}

	/// Try to guess a file's mime type from its extension.
	pub fn from_extension(ext: &str) -> Option<Self> {
		Some(match ext {
			"flac" => Self::Flac,
			"mp3" => Self::Mp3,
			_ => return None,
		})
	}

	/// Get the extension we use for files with this type.
	/// Includes a dot. Might be the empty string.
	pub fn extension(&self) -> &str {
		match self {
			Self::Blob => "",
			Self::Unknown(_) => "",

			Self::Flac => ".flac",
			Self::Mp3 => ".mp3",
			Self::Jpg => ".jpg",
			Self::Png => ".png",
		}
	}
}

impl Display for MimeType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.to_db_str())
	}
}
