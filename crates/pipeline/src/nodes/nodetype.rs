use serde::Deserialize;
use serde_with::serde_as;
use ufo_audiofile::common::tagtype::TagType;
use ufo_util::data::{PipelineData, PipelineDataType};

use crate::{portspec::PipelinePortSpec, syntax::labels::PipelinePortLabel};

use super::{
	nodeinstance::PipelineNodeInstance,
	tags::{extractcovers::ExtractCovers, extracttags::ExtractTags, striptags::StripTags},
	util::ifnone::IfNone,
};

#[serde_as]
#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
#[serde(deny_unknown_fields)]
pub enum PipelineNodeType {
	/// The pipeline's outputs.
	/// This cannot be created by a user;
	/// and EXACTLY one must exist in every pipeline.
	///
	/// Note that pipeline outputs provide *inputs* inside the pipeline.
	#[serde(skip_deserializing)]
	PipelineOutputs {
		inputs: Vec<(PipelinePortLabel, PipelineDataType)>,
	},

	/// The pipeline's inputs.
	/// This cannot be created by a user;
	/// and EXACTLY one must exist in every pipeline.
	///
	/// Note that pipeline inputs provide *outputs* inside the pipeline.
	#[serde(skip_deserializing)]
	PipelineInputs {
		outputs: Vec<(PipelinePortLabel, PipelineDataType)>,
	},

	/// A node that provides a constant value.
	/// These can only be created as inline nodes.
	#[serde(skip_deserializing)]
	ConstantNode {
		value: PipelineData,
	},

	ExtractTags {
		tags: Vec<TagType>,
	},

	ExtractCovers,
	StripTags,
	IfNone,
}

impl PipelineNodeType {
	pub fn build(&self, name: &str) -> PipelineNodeInstance {
		match self {
			PipelineNodeType::ConstantNode { .. } => PipelineNodeInstance::ConstantNode {
				node_type: self.clone(),
			},
			PipelineNodeType::PipelineOutputs { .. } => PipelineNodeInstance::PipelineOutputs {
				node_type: self.clone(),
			},
			PipelineNodeType::PipelineInputs { .. } => PipelineNodeInstance::PipelineInputs {
				node_type: self.clone(),
			},
			PipelineNodeType::IfNone => PipelineNodeInstance::IfNone {
				node_type: self.clone(),
				name: name.into(),
				node: IfNone::new(),
			},
			PipelineNodeType::StripTags => PipelineNodeInstance::StripTags {
				node_type: self.clone(),
				name: name.into(),
				node: StripTags::new(),
			},
			PipelineNodeType::ExtractTags { tags } => PipelineNodeInstance::ExtractTags {
				node_type: self.clone(),
				name: name.into(),
				node: ExtractTags::new(tags.clone()),
			},
			PipelineNodeType::ExtractCovers => PipelineNodeInstance::ExtractCovers {
				node_type: self.clone(),
				name: name.into(),
				node: ExtractCovers::new(),
			},
		}
	}
}

impl PipelineNodeType {
	pub fn inputs(&self) -> PipelinePortSpec {
		match self {
			Self::PipelineOutputs { inputs, .. } => PipelinePortSpec::Vec(inputs),
			Self::PipelineInputs { .. } => PipelinePortSpec::Static(&[]),
			Self::ConstantNode { .. } => PipelinePortSpec::Static(&[]),
			Self::ExtractTags { .. } => {
				PipelinePortSpec::Static(&[("data", PipelineDataType::Binary)])
			}
			Self::IfNone => PipelinePortSpec::Static(&[
				("data", PipelineDataType::Text),
				("ifnone", PipelineDataType::Text),
			]),
			Self::StripTags => PipelinePortSpec::Static(&[("data", PipelineDataType::Binary)]),
			Self::ExtractCovers => PipelinePortSpec::Static(&[("data", PipelineDataType::Binary)]),
		}
	}

	pub fn outputs(&self) -> PipelinePortSpec {
		match self {
			Self::PipelineOutputs { .. } => PipelinePortSpec::Static(&[]),
			Self::PipelineInputs { outputs, .. } => PipelinePortSpec::Vec(outputs),
			Self::ConstantNode { value } => {
				PipelinePortSpec::VecOwned(vec![("out".into(), value.get_type())])
			}
			Self::ExtractTags { tags } => PipelinePortSpec::VecOwned(
				tags.iter()
					.map(|x| (Into::<&str>::into(x).into(), PipelineDataType::Text))
					.collect(),
			),
			Self::IfNone => PipelinePortSpec::Static(&[("out", PipelineDataType::Text)]),
			Self::StripTags => PipelinePortSpec::Static(&[("out", PipelineDataType::Binary)]),
			Self::ExtractCovers => PipelinePortSpec::Static(&[]),
		}
	}
}
