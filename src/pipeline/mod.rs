use std::{collections::HashMap, error::Error, fmt::Display, str::FromStr};

use serde::Deserialize;

use self::nodes::PipelineNodes;
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

#[derive(Debug, Deserialize)]
pub enum PipelineDataType {
	None,
	Text,
	Binary,
}

// TODO: error
impl FromStr for PipelineDataType {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"none" => Ok(Self::None),
			"text" => Ok(Self::Text),
			"binary" => Ok(Self::Binary),
			_ => Err("bad data type".to_string()),
		}
	}
}

// TODO: enforce docs
// TODO: node id, port id type
#[derive(Debug, Hash, PartialEq, Eq)]
struct OutputLink {
	node: String,
	port: String,
}

#[derive(Debug, Hash, PartialEq, Eq)]
struct InputLink {
	node: String,
	port: String,
}

struct Pipeline {
	inputs: HashMap<String, ItemType>,
	nodes: HashMap<String, PipelineNodes>,
	links: HashMap<InputLink, OutputLink>,
}
