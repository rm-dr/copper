//! Media type utilities

use std::{fmt::Display, str::FromStr};

use serde_with::{DeserializeFromStr, SerializeDisplay};
use tracing::warn;
use utoipa::ToSchema;

/// A media type, conveniently parsed
#[allow(missing_docs)]
#[derive(Debug, PartialEq, Eq, Clone, SerializeDisplay, DeserializeFromStr, ToSchema)]
pub enum MimeType {
	/// A mimetype we didn't recognize
	Unknown(String),

	/// An unstructured binary blob
	Blob,

	// Images
	Png,
	Jpg,

	// Audo
	Flac,
	Mp3,
}

impl FromStr for MimeType {
	// Must match `display` below, but may provide other alternatives.

	type Err = std::convert::Infallible;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"application/octet-stream" => Self::Blob,
			"image/png" => Self::Png,
			"image/jpg" => Self::Jpg,
			"image/jpeg" => Self::Jpg,
			"audio/flac" => Self::Flac,
			_ => {
				warn!(message = "Encountered unknown mimetype", mime_string = s);
				Self::Unknown(s.into())
			}
		})
	}
}

impl Display for MimeType {
	/// Get a string representation of this mimetype.
	///
	/// The following always holds
	/// ```no_run
	/// // x: MimeType
	/// MimeType::from(x.to_string()) == x
	/// ```
	///
	/// The following might not hold:
	/// ```no_run
	/// // y: &str
	/// MimeType::from(y).to_string() == y
	/// ```
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Blob => write!(f, "application/octet-stream"),
			Self::Png => write!(f, "image/png"),
			Self::Jpg => write!(f, "image/jpeg"),
			Self::Flac => write!(f, "audio/flac"),
			Self::Mp3 => write!(f, "audio/mp3"),
			Self::Unknown(x) => write!(f, "{}", x),
		}
	}
}

impl From<String> for MimeType {
	fn from(value: String) -> Self {
		Self::from_str(&value).unwrap()
	}
}

impl From<&str> for MimeType {
	fn from(value: &str) -> Self {
		Self::from_str(value).unwrap()
	}
}

impl MimeType {
	// Must match `From<String>` above

	/// Try to guess a file's mime type from its extension.
	/// `ext` should NOT start with a dot.
	pub fn from_extension(ext: &str) -> Option<Self> {
		Some(match ext {
			"flac" => Self::Flac,
			"mp3" => Self::Mp3,
			_ => {
				warn!(
					message = "Could not determine mime type from extension",
					extension = ext
				);
				return None;
			}
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
