use std::{fmt::Display, str::FromStr};

use serde_with::{DeserializeFromStr, SerializeDisplay};
use tracing::warn;

/// A media type, conveniently parsed
#[allow(missing_docs)]
#[derive(Debug, PartialEq, Eq, Clone, SerializeDisplay, DeserializeFromStr)]
pub enum MimeType {
	// We INTENTIONALLY do not implement `ToSchema` on MimeType, since it generates a bad impl.
	// Instead, we use #[schema(value_type = String)] on any mimetype fields.
	// TODO: manually implement ToSchema here.
	/// A mimetype we didn't recognize
	Other(String),

	/// An unstructured binary blob
	/// Use this whenever a mime type is unknown
	Blob,

	// Images
	Png,
	Jpg,
	Gif,
	Avif,

	// Audio
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
			"image/gif" => Self::Gif,
			"image/avif" => Self::Avif,
			"audio/flac" => Self::Flac,
			_ => {
				warn!(message = "Encountered unknown mimetype", mime_string = s);
				Self::Other(s.into())
			}
		})
	}
}

impl Display for MimeType {
	/// Get a string representation of this mimetype.
	///
	/// The following always holds
	/// ```notrust
	/// // x: MimeType
	/// MimeType::from(x.to_string()) == x
	/// ```
	///
	/// The following might not hold:
	/// ```notrust
	/// // y: &str
	/// MimeType::from(y).to_string() == y
	/// ```
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Blob => write!(f, "application/octet-stream"),

			Self::Png => write!(f, "image/png"),
			Self::Jpg => write!(f, "image/jpeg"),
			Self::Gif => write!(f, "image/gif"),
			Self::Avif => write!(f, "image/avif"),

			Self::Flac => write!(f, "audio/flac"),
			Self::Mp3 => write!(f, "audio/mp3"),
			Self::Other(x) => write!(f, "{}", x),
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
			"gif" => Self::Gif,
			"avif" => Self::Avif,
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
			Self::Other(_) => "",

			Self::Flac => ".flac",
			Self::Mp3 => ".mp3",
			Self::Jpg => ".jpg",
			Self::Png => ".png",
			Self::Gif => ".gif",
			Self::Avif => ".avif",
		}
	}
}
