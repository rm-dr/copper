use serde_with::DeserializeFromStr;
use smartstring::{LazyCompact, SmartString};
use std::{collections::HashMap, str::FromStr};

use super::{PipelineData, PipelineDataType, PipelineError};

pub mod ifnone;
pub mod tags;

pub trait PipelineNode {
	fn run(
		inputs: HashMap<SmartString<LazyCompact>, PipelineData>,
	) -> Result<HashMap<SmartString<LazyCompact>, PipelineData>, PipelineError>;

	/// List this node's inputs.
	/// This is a list of ("output name", output type)
	/// Input names MUST be unique. This is not enforced!
	fn get_inputs() -> &'static [(&'static str, PipelineDataType)];

	/// List this node's outputs.
	/// This is a list of ("output name", output type)
	/// Output names MUST be unique. This is not enforced!
	fn get_outputs() -> &'static [(&'static str, PipelineDataType)];
}

#[derive(Debug, Clone, Copy, DeserializeFromStr)]
pub enum PipelineNodes {
	ExtractTag,
	IfNone,
}

impl PipelineNodes {
	pub fn run(
		&self,
		inputs: HashMap<SmartString<LazyCompact>, PipelineData>,
	) -> Result<HashMap<SmartString<LazyCompact>, PipelineData>, PipelineError> {
		match self {
			Self::ExtractTag => tags::ExtractTag::run(inputs),
			Self::IfNone => ifnone::IfNone::run(inputs),
		}
	}

	pub fn get_inputs(&self) -> &'static [(&'static str, PipelineDataType)] {
		match self {
			Self::ExtractTag => tags::ExtractTag::get_inputs(),
			Self::IfNone => ifnone::IfNone::get_inputs(),
		}
	}

	pub fn get_outputs(&self) -> &'static [(&'static str, PipelineDataType)] {
		match self {
			Self::ExtractTag => tags::ExtractTag::get_outputs(),
			Self::IfNone => ifnone::IfNone::get_outputs(),
		}
	}
}

// TODO: better error
impl FromStr for PipelineNodes {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"ExtractTag" => Ok(Self::ExtractTag),
			"IfNone" => Ok(Self::IfNone),
			_ => Err("bad node type".to_string()),
		}
	}
}
