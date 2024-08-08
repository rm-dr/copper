use std::{error::Error, fmt::Display, str::FromStr};

use serde::Deserialize;
use serde_with::DeserializeFromStr;
use smartstring::{LazyCompact, SmartString};

use crate::model::ItemType;

pub mod nodes;

// Pub is only for testing, remove.
pub mod syntax;

#[derive(Debug)]
pub enum PipelineError {
	FileSystemError(Box<dyn Error>),
	UnsupportedDataType,
}

// TODO: clean up
impl Error for PipelineError {}
unsafe impl Send for PipelineError {}
unsafe impl Sync for PipelineError {}
impl Display for PipelineError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::FileSystemError(e) => write!(f, "Fs error: {e}"),
			Self::UnsupportedDataType => write!(f, "Unsupported Item data type"),
		}
	}
}

#[derive(Debug)]
pub enum PipelineData {
	None,
	Text(String),

	// TODO: Stream data?
	// Also, no clone.
	Binary { data_type: ItemType, data: Vec<u8> },
}

#[derive(Debug, DeserializeFromStr, PartialEq, Eq)]
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

// TODO: enforce docs
// TODO: node id, port id type
#[derive(Debug, Hash, PartialEq, Eq, Deserialize, Clone)]
pub struct PortLink {
	node: SmartString<LazyCompact>,
	port: SmartString<LazyCompact>,
}

impl Display for PortLink {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}.{}", self.node, self.port)
	}
}
