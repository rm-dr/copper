use serde::Deserialize;
use serde_with::serde_as;
use ufo_audiofile::common::tagtype::TagType;

use crate::{
	data::{PipelineData, PipelineDataType},
	input::PipelineInputKind,
	output::PipelineOutputKind,
	portspec::PipelinePortSpec,
	syntax::labels::PipelinePortLabel,
};

use super::{
	nodeinstance::PipelineNodeInstance,
	tags::{extractcovers::ExtractCovers, extracttags::ExtractTags, striptags::StripTags},
	util::{constant::Constant, hash::Hash, ifnone::IfNone, noop::Noop},
};

#[serde_as]
#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
#[serde(deny_unknown_fields)]
pub enum PipelineNodeType {
	// Magic nodes
	/// The pipeline's outputs.
	/// This cannot be created by a user;
	/// and EXACTLY one must exist in every pipeline.
	///
	/// Note that pipeline outputs provide *inputs* inside the pipeline.
	#[serde(skip_deserializing)]
	PipelineOutputs {
		pipeline: String,
		output_type: PipelineOutputKind,
		inputs: Vec<(PipelinePortLabel, PipelineDataType)>,
	},

	/// The pipeline's inputs.
	/// This cannot be created by a user;
	/// and EXACTLY one must exist in every pipeline.
	///
	/// Note that pipeline inputs provide *outputs* inside the pipeline.
	#[serde(skip_deserializing)]
	PipelineInputs {
		input_type: PipelineInputKind,
		outputs: Vec<(PipelinePortLabel, PipelineDataType)>,
	},

	/// A node that provides a constant value.
	/// These can only be created as inline nodes.
	#[serde(skip_deserializing)]
	ConstantNode {
		value: PipelineData,
	},

	/// A node that invokes another pipeline.
	/// This will never appear in a fully prepared pipeline graph,
	/// since it is replaced with the given pipeline's contents.
	Pipeline {
		pipeline: String,
	},

	// Utility nodes
	IfNone,
	Hash,
	Noop {
		inputs: Vec<(PipelinePortLabel, PipelineDataType)>,
	},

	// Audio nodes
	ExtractTags {
		tags: Vec<TagType>,
	},
	ExtractCovers,
	StripTags,
}

impl PipelineNodeType {
	pub fn build(&self, name: &str) -> PipelineNodeInstance {
		match self {
			// Magic
			PipelineNodeType::Pipeline { .. } => unreachable!(),
			PipelineNodeType::ConstantNode { value } => PipelineNodeInstance::Constant {
				node_type: self.clone(),
				node: Constant::new(value.clone()),
			},
			PipelineNodeType::PipelineOutputs { .. } => PipelineNodeInstance::PipelineOutputs {
				node_type: self.clone(),
			},
			PipelineNodeType::PipelineInputs { .. } => PipelineNodeInstance::PipelineInputs {
				node_type: self.clone(),
			},

			// Util
			PipelineNodeType::IfNone => PipelineNodeInstance::IfNone {
				node_type: self.clone(),
				name: name.into(),
				node: IfNone::new(),
			},
			PipelineNodeType::Noop { inputs } => PipelineNodeInstance::Noop {
				node_type: self.clone(),
				name: name.into(),
				node: Noop::new(inputs.clone()),
			},
			PipelineNodeType::Hash => PipelineNodeInstance::Hash {
				node_type: self.clone(),
				name: name.into(),
				node: Hash::new(),
			},

			// Audio
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
			// Magic
			Self::Pipeline { .. } => unreachable!(),
			Self::PipelineOutputs { inputs, .. } => PipelinePortSpec::Vec(inputs),
			Self::PipelineInputs { input_type, .. } => input_type.get_inputs(),
			Self::ConstantNode { .. } => PipelinePortSpec::Static(&[]),

			// Util
			Self::IfNone => PipelinePortSpec::Static(&[
				("data", PipelineDataType::Text),
				("ifnone", PipelineDataType::Text),
			]),
			Self::Noop { inputs } => PipelinePortSpec::Vec(inputs),
			Self::Hash => PipelinePortSpec::Static(&[("data", PipelineDataType::Binary)]),

			// Audio
			Self::ExtractTags { .. } => {
				PipelinePortSpec::Static(&[("data", PipelineDataType::Binary)])
			}
			Self::StripTags => PipelinePortSpec::Static(&[("data", PipelineDataType::Binary)]),
			Self::ExtractCovers => PipelinePortSpec::Static(&[("data", PipelineDataType::Binary)]),
		}
	}

	pub fn outputs(&self) -> PipelinePortSpec {
		match self {
			// Magic
			Self::Pipeline { .. } => unreachable!(),
			Self::PipelineOutputs { output_type, .. } => output_type.get_outputs(),
			Self::PipelineInputs { outputs, .. } => PipelinePortSpec::Vec(outputs),
			Self::ConstantNode { value } => {
				PipelinePortSpec::VecOwned(vec![("out".into(), value.get_type())])
			}

			// Util
			Self::IfNone => PipelinePortSpec::Static(&[("out", PipelineDataType::Text)]),
			Self::Hash => PipelinePortSpec::Static(&[("hash", PipelineDataType::Text)]),
			Self::Noop { inputs } => PipelinePortSpec::Vec(inputs),

			// Audio
			Self::ExtractTags { tags } => PipelinePortSpec::VecOwned(
				tags.iter()
					.map(|x| (Into::<&str>::into(x).into(), PipelineDataType::Text))
					.collect(),
			),
			Self::StripTags => PipelinePortSpec::Static(&[("out", PipelineDataType::Binary)]),
			Self::ExtractCovers => {
				PipelinePortSpec::Static(&[("cover_data", PipelineDataType::Binary)])
			}
		}
	}

	pub fn is_pipeline_input(&self) -> bool {
		matches!(self, Self::PipelineInputs { .. })
	}
}
