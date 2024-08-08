use std::{collections::HashMap, str::FromStr};

use serde::Deserialize;

use super::{PipelineData, PipelineError};

pub mod ifnone;
pub mod tags;

pub trait PipelineNode {
	fn run(
		inputs: HashMap<String, PipelineData>,
	) -> Result<HashMap<String, PipelineData>, PipelineError>;
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum PipelineNodes {
	ExtractTag,
	IfNone,
}

impl PipelineNodes {
	pub fn run(
		&self,
		inputs: HashMap<String, PipelineData>,
	) -> Result<HashMap<String, PipelineData>, PipelineError> {
		match self {
			Self::ExtractTag => tags::ExtractTag::run(inputs),
			Self::IfNone => ifnone::IfNone::run(inputs),
		}
	}
}

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
