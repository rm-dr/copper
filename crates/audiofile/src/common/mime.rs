use smartstring::{LazyCompact, SmartString};

#[derive(Debug, PartialEq, Eq)]
pub enum MimeType {
	Unknown(SmartString<LazyCompact>),
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
