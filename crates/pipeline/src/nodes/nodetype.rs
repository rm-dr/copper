use serde::Deserialize;
use smartstring::{LazyCompact, SmartString};
use std::str::FromStr;
use ufo_util::data::PipelineDataType;

use crate::portspec::PipelinePortSpec;

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
	pub fn outputs(&self) -> PipelinePortSpec {
		match self {
			PipelineNodeType::ExtractTags => PipelinePortSpec::Static(&[
				("title", PipelineDataType::Text),
				("album", PipelineDataType::Text),
				("artist", PipelineDataType::Text),
				("genre", PipelineDataType::Text),
				("comment", PipelineDataType::Text),
				("track", PipelineDataType::Text),
				("disk", PipelineDataType::Text),
				("disk_total", PipelineDataType::Text),
				("year", PipelineDataType::Text),
			]),
			PipelineNodeType::IfNone => {
				PipelinePortSpec::Static(&[("out", PipelineDataType::Text)])
			}
		}
	}

	pub fn inputs(&self) -> PipelinePortSpec {
		match self {
			Self::ExtractTags => PipelinePortSpec::Static(&[("data", PipelineDataType::Binary)]),
			Self::IfNone => PipelinePortSpec::Static(&[
				("data", PipelineDataType::Text),
				("ifnone", PipelineDataType::Text),
			]),
		}
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
