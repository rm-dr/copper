use serde::Deserialize;
use smartstring::{LazyCompact, SmartString};
use std::{collections::HashMap, str::FromStr};

use super::{
	components::PipelinePortLabel,
	data::{PipelineData, PipelineDataType},
	errors::PipelineError,
};

pub mod ifnone;
pub mod tags;

pub trait PipelineNode {
	fn run(
		inputs: HashMap<PipelinePortLabel, Option<PipelineData>>,
	) -> Result<HashMap<PipelinePortLabel, Option<PipelineData>>, PipelineError>;

	/// List the inputs this node provides.
	/// Input names MUST be unique. This is not enforced!
	fn get_inputs() -> impl Iterator<Item = PipelinePortLabel>;

	/// List the outputs this node provides.
	/// Output names MUST be unique. This is not enforced!
	fn get_outputs() -> impl Iterator<Item = PipelinePortLabel>;

	/// Does this pipeline provide the given input port?
	/// If it does, return its type. If it doesn't, return None.
	fn get_input(input: &PipelinePortLabel) -> Option<PipelineDataType>;

	/// Does this pipeline provide the given output port?
	/// If it does, return its type. If it doesn't, return None.
	fn get_output(output: &PipelinePortLabel) -> Option<PipelineDataType>;
}

#[derive(Debug, Clone, Copy)]
pub enum PipelineNodes {
	ExtractTag,
	IfNone,
}

impl PipelineNodes {
	pub fn run(
		&self,
		inputs: HashMap<PipelinePortLabel, Option<PipelineData>>,
	) -> Result<HashMap<PipelinePortLabel, Option<PipelineData>>, PipelineError> {
		match self {
			Self::ExtractTag => tags::ExtractTag::run(inputs),
			Self::IfNone => ifnone::IfNone::run(inputs),
		}
	}

	pub fn get_inputs(&self) -> Box<dyn Iterator<Item = PipelinePortLabel>> {
		match self {
			Self::ExtractTag => Box::new(tags::ExtractTag::get_inputs()),
			Self::IfNone => Box::new(ifnone::IfNone::get_inputs()),
		}
	}

	pub fn get_outputs(&self) -> Box<dyn Iterator<Item = PipelinePortLabel>> {
		match self {
			Self::ExtractTag => Box::new(tags::ExtractTag::get_outputs()),
			Self::IfNone => Box::new(ifnone::IfNone::get_outputs()),
		}
	}

	pub fn get_input(&self, input: &PipelinePortLabel) -> Option<PipelineDataType> {
		match self {
			Self::ExtractTag => tags::ExtractTag::get_input(input),
			Self::IfNone => ifnone::IfNone::get_input(input),
		}
	}

	pub fn get_output(&self, output: &PipelinePortLabel) -> Option<PipelineDataType> {
		match self {
			Self::ExtractTag => tags::ExtractTag::get_output(output),
			Self::IfNone => ifnone::IfNone::get_output(output),
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

impl<'de> Deserialize<'de> for PipelineNodes {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		let addr_str = SmartString::<LazyCompact>::deserialize(deserializer)?;
		let s = Self::from_str(&addr_str);
		s.map_err(serde::de::Error::custom)
	}
}
