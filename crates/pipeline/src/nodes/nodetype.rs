use serde::Deserialize;
use smartstring::{LazyCompact, SmartString};
use std::str::FromStr;
use ufo_util::data::PipelineDataType;

use super::{ifnone::IfNone, nodeinstance::PipelineNodeInstance, tags::ExtractTags};

#[derive(Debug, Clone, Copy)]
pub enum PipelineNodeType {
	ExtractTags,
	IfNone,
}

impl PipelineNodeType {
	pub fn build(self, name: &str) -> PipelineNodeInstance {
		match self {
			PipelineNodeType::IfNone => PipelineNodeInstance::IfNone {
				name: name.into(),
				node: IfNone::new(),
			},
			PipelineNodeType::ExtractTags => PipelineNodeInstance::ExtractTags {
				name: name.into(),
				node: ExtractTags::new(),
			},
		}
	}
}

impl PipelineNodeType {
	pub fn n_outputs(&self) -> usize {
		match self {
			PipelineNodeType::ExtractTags => 9,
			PipelineNodeType::IfNone => 1,
		}
	}

	pub fn output_name(&self, output: usize) -> String {
		match self {
			PipelineNodeType::ExtractTags => [
				"title",
				"album",
				"artist",
				"genre",
				"comment",
				"track",
				"disk",
				"disk_total",
				"year",
			]
			.get(output),

			PipelineNodeType::IfNone => ["out"].get(output),
		}
		.unwrap()
		.to_string()
	}

	pub fn output_type(&self, output: usize) -> PipelineDataType {
		*match self {
			PipelineNodeType::ExtractTags => [
				PipelineDataType::Text,
				PipelineDataType::Text,
				PipelineDataType::Text,
				PipelineDataType::Text,
				PipelineDataType::Text,
				PipelineDataType::Text,
				PipelineDataType::Text,
				PipelineDataType::Text,
				PipelineDataType::Text,
			]
			.get(output),

			PipelineNodeType::IfNone => [PipelineDataType::Text].get(output),
		}
		.unwrap()
	}

	pub fn output_with_name(&self, name: &str) -> Option<usize> {
		(0..self.n_outputs()).find(|x| self.output_name(*x) == name)
	}

	pub fn n_inputs(&self) -> usize {
		match self {
			Self::ExtractTags => 1,
			Self::IfNone => 2,
		}
	}

	pub fn input_name(&self, input: usize) -> String {
		match self {
			Self::ExtractTags => ["data"].get(input),
			Self::IfNone => ["data", "ifnone"].get(input),
		}
		.unwrap()
		.to_string()
	}

	pub fn input_type(&self, input: usize) -> PipelineDataType {
		*match self {
			Self::ExtractTags => [PipelineDataType::Binary].get(input),
			Self::IfNone => [PipelineDataType::Text, PipelineDataType::Text].get(input),
		}
		.unwrap()
	}

	pub fn input_with_name(&self, name: &str) -> Option<usize> {
		(0..self.n_inputs()).find(|x| self.input_name(*x) == name)
	}
}

// TODO: better error
impl FromStr for PipelineNodeType {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"ExtractTag" => Ok(Self::ExtractTags),
			"IfNone" => Ok(Self::IfNone),
			_ => Err("bad node type".to_string()),
		}
	}
}

impl<'de> Deserialize<'de> for PipelineNodeType {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		let addr_str = SmartString::<LazyCompact>::deserialize(deserializer)?;
		let s = Self::from_str(&addr_str);
		s.map_err(serde::de::Error::custom)
	}
}
