use serde::Deserialize;
use serde_with::serde_as;
use ufo_audiofile::common::tagtype::TagType;
use ufo_util::data::PipelineDataType;

use crate::portspec::PipelinePortSpec;

use super::{ifnone::IfNone, nodeinstance::PipelineNodeInstance, tags::ExtractTags, StripTags};

#[serde_as]
#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
#[serde(deny_unknown_fields)]
pub enum PipelineNodeType {
	ExtractTags { tags: Vec<TagType> },
	StripTags,
	IfNone,
}

impl PipelineNodeType {
	pub fn build(&self, name: &str) -> PipelineNodeInstance {
		match self {
			Self::IfNone => PipelineNodeInstance::IfNone {
				name: name.into(),
				node: IfNone::new(),
			},
			Self::StripTags => PipelineNodeInstance::StripTags {
				name: name.into(),
				node: StripTags::new(),
			},
			Self::ExtractTags { tags } => PipelineNodeInstance::ExtractTags {
				name: name.into(),
				node: ExtractTags::new(tags.clone()),
			},
		}
	}
}

impl PipelineNodeType {
	pub fn outputs(&self) -> PipelinePortSpec {
		match self {
			Self::ExtractTags { tags } => PipelinePortSpec::VecOwned(
				tags.iter()
					.map(|x| (Into::<&str>::into(x).into(), PipelineDataType::Text))
					.collect(),
			),
			Self::IfNone => PipelinePortSpec::Static(&[("out", PipelineDataType::Text)]),
			Self::StripTags => PipelinePortSpec::Static(&[("out", PipelineDataType::Binary)]),
		}
	}

	pub fn inputs(&self) -> PipelinePortSpec {
		match self {
			Self::ExtractTags { .. } => {
				PipelinePortSpec::Static(&[("data", PipelineDataType::Binary)])
			}
			Self::IfNone => PipelinePortSpec::Static(&[
				("data", PipelineDataType::Text),
				("ifnone", PipelineDataType::Text),
			]),
			Self::StripTags => PipelinePortSpec::Static(&[("data", PipelineDataType::Binary)]),
		}
	}
}
