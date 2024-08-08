use serde::Deserialize;
use smartstring::{LazyCompact, SmartString};
use std::{fmt::Debug, str::FromStr};

// TODO: binary format contains data?
// TODO: Stream data?
// TODO: no clone vec

/// What kind of audio data is this?
#[derive(Debug, Copy, Clone)]
pub enum AudioFormat {
	Mp3,
	Flac,
	//Ogg,
}

/// How should this binary data be interpreted?
#[derive(Debug, Clone, Copy)]
pub enum BinaryFormat {
	/// A plain binary blob
	Blob,

	/// Plain text
	// Text,

	/// An audio file
	Audio(AudioFormat),
}

/// A bit of data inside a pipeline.
/// These are instances of [`PipelineDataType`].
#[derive(Clone)]
pub enum PipelineData {
	Text(String),
	Binary { format: BinaryFormat, data: Vec<u8> },
}

impl Debug for PipelineData {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Text(s) => write!(f, "Text({})", s),
			Self::Binary { format, .. } => write!(f, "Binary({:?})", format),
		}
	}
}

/// A data type inside a pipeline.
/// Corresponds to [`PipelineData`]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PipelineDataType {
	Text,
	Binary,
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
