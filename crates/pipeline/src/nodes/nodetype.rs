use std::sync::Arc;

use serde::Deserialize;
use serde_with::serde_as;
use ufo_audiofile::common::tagtype::TagType;
use ufo_util::data::{PipelineData, PipelineDataType};

use crate::portspec::PipelinePortSpec;

use super::{
	nodeinstance::PipelineNodeInstance,
	tags::{striptags::StripTags, tags::ExtractTags},
	util::ifnone::IfNone,
};

#[serde_as]
#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
#[serde(deny_unknown_fields)]
pub enum PipelineNodeType {
	/// The pipeline's "external" node.
	/// This cannot be created by a user;
	/// and EXACTLY one must exist in every pipeline.
	///
	/// This is handled by the `syntax` module.
	#[serde(skip_deserializing)]
	ExternalNode,

	/// A node that provides a constant value.
	/// These can only be created as inline nodes.
	#[serde(skip_deserializing)]
	ConstantNode {
		data: Arc<PipelineData>,
	},

	ExtractTags {
		tags: Vec<TagType>,
	},
	StripTags,
	IfNone,
}

impl PipelineNodeType {
	pub fn build(&self, name: &str) -> PipelineNodeInstance {
		match self {
			Self::ConstantNode { data } => PipelineNodeInstance::ConstantNode(data.clone()),
			Self::ExternalNode => PipelineNodeInstance::ExternalNode,
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
			Self::ExternalNode => PipelinePortSpec::Static(&[]),
			Self::ConstantNode { data } => {
				PipelinePortSpec::VecOwned(vec![("out".into(), data.as_ref().get_type())])
			}
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
			Self::ExternalNode => PipelinePortSpec::Static(&[]),
			Self::ConstantNode { .. } => PipelinePortSpec::Static(&[]),
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
