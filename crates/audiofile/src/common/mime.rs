//! MIME type convenience structures

use smartstring::{LazyCompact, SmartString};

/// A MIME type, conveniently parsed.
#[derive(Debug, PartialEq, Eq)]
pub enum MimeType {
	/// A mimetype we didn't recognize
	Unknown(SmartString<LazyCompact>),

	/// A png image
	Png,
}

impl From<String> for MimeType {
	fn from(value: String) -> Self {
		match &value[..] {
			"image/png" => Self::Png,
			_ => Self::Unknown(value.into()),
		}
	}
}
