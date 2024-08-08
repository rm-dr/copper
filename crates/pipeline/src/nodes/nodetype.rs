use serde::Deserialize;
use smartstring::{LazyCompact, SmartString};
use std::str::FromStr;
use ufo_util::data::PipelineDataType;

use crate::syntax::labels::PipelinePortLabel;

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
	// TODO: efficiency. Don't allocate a new vec here.
	pub fn outputs(&self) -> Vec<(PipelinePortLabel, PipelineDataType)> {
		match self {
			PipelineNodeType::ExtractTags => vec![
				("title".into(), PipelineDataType::Text),
				("album".into(), PipelineDataType::Text),
				("artist".into(), PipelineDataType::Text),
				("genre".into(), PipelineDataType::Text),
				("comment".into(), PipelineDataType::Text),
				("track".into(), PipelineDataType::Text),
				("disk".into(), PipelineDataType::Text),
				("disk_total".into(), PipelineDataType::Text),
				("year".into(), PipelineDataType::Text),
			],
			PipelineNodeType::IfNone => vec![("out".into(), PipelineDataType::Text)],
		}
	}

	// TODO: efficiency. Don't allocate a new vec here.
	pub fn inputs(&self) -> Vec<(PipelinePortLabel, PipelineDataType)> {
		match self {
			Self::ExtractTags => vec![("data".into(), PipelineDataType::Binary)],
			Self::IfNone => vec![
				("data".into(), PipelineDataType::Text),
				("ifnone".into(), PipelineDataType::Text),
			],
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
