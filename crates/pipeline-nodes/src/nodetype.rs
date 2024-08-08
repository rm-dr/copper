use std::sync::Arc;

use serde::Deserialize;
use serde_with::serde_as;
use smartstring::{LazyCompact, SmartString};
use ufo_audiofile::common::tagtype::TagType;
use ufo_pipeline::{
	data::{PipelineData, PipelineDataType},
	node::{PipelineNode, PipelineNodeStub},
	portspec::PipelinePortSpec,
	syntax::labels::PipelinePortLabel,
};
use ufo_storage::api::{ClassHandle, Dataset};

use crate::{input::file::FileInput, output::storage::StorageOutput};

use super::{
	nodeinstance::UFONodeInstance,
	tags::{extractcovers::ExtractCovers, extracttags::ExtractTags, striptags::StripTags},
	util::{constant::Constant, hash::Hash, ifnone::IfNone, noop::Noop, print::Print},
};

#[serde_as]
#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
#[serde(deny_unknown_fields)]
pub enum UFONodeType {
	/// A node that provides a constant value.
	#[serde(skip_deserializing)]
	ConstantNode {
		value: PipelineData,
	},

	// Utility nodes
	IfNone,
	Hash,
	Print,
	Noop {
		#[serde(rename = "input")]
		#[serde_as(as = "serde_with::Map<_, _>")]
		inputs: Vec<(PipelinePortLabel, PipelineDataType)>,
	},

	// Audio nodes
	ExtractTags {
		tags: Vec<TagType>,
	},
	ExtractCovers,
	StripTags,

	File,
	Dataset {
		class: String,
		#[serde(rename = "attr")]
		#[serde_as(as = "serde_with::Map<_, _>")]
		attrs: Vec<(SmartString<LazyCompact>, PipelineDataType)>,
	},
}

impl PipelineNodeStub for UFONodeType {
	type NodeType = UFONodeInstance;

	fn build(
		&self,
		ctx: Arc<<Self::NodeType as PipelineNode>::RunContext>,
		name: &str,
	) -> UFONodeInstance {
		match self {
			// Magic
			UFONodeType::ConstantNode { value } => UFONodeInstance::Constant {
				node_type: self.clone(),
				node: Constant::new(value.clone()),
			},

			// Util
			UFONodeType::IfNone => UFONodeInstance::IfNone {
				node_type: self.clone(),
				name: name.into(),
				node: IfNone::new(),
			},
			UFONodeType::Noop { inputs } => UFONodeInstance::Noop {
				node_type: self.clone(),
				name: name.into(),
				node: Noop::new(inputs.clone()),
			},
			UFONodeType::Print => UFONodeInstance::Print {
				node_type: self.clone(),
				name: name.into(),
				node: Print::new(),
			},
			UFONodeType::Hash => UFONodeInstance::Hash {
				node_type: self.clone(),
				name: name.into(),
				node: Hash::new(),
			},

			// Audio
			UFONodeType::StripTags => UFONodeInstance::StripTags {
				node_type: self.clone(),
				name: name.into(),
				node: StripTags::new(),
			},
			UFONodeType::ExtractTags { tags } => UFONodeInstance::ExtractTags {
				node_type: self.clone(),
				name: name.into(),
				node: ExtractTags::new(tags.clone()),
			},
			UFONodeType::ExtractCovers => UFONodeInstance::ExtractCovers {
				node_type: self.clone(),
				name: name.into(),
				node: ExtractCovers::new(),
			},

			UFONodeType::File => UFONodeInstance::File {
				node_type: self.clone(),
				name: name.into(),
				node: FileInput::new(),
			},
			UFONodeType::Dataset { class, attrs } => {
				let d = ctx.dataset.lock().unwrap();
				let class = d.get_class(class).unwrap().unwrap();

				UFONodeInstance::Dataset {
					node_type: self.clone(),
					name: name.into(),
					node: StorageOutput::new(class.clone(), attrs.clone()),
				}
			}
		}
	}

	fn inputs(&self, _ctx: Arc<<Self::NodeType as PipelineNode>::RunContext>) -> PipelinePortSpec {
		match self {
			// Util
			Self::ConstantNode { .. } => PipelinePortSpec::Static(&[]),
			Self::IfNone => PipelinePortSpec::Static(&[
				("data", PipelineDataType::Text),
				("ifnone", PipelineDataType::Text),
			]),
			Self::Noop { inputs } => PipelinePortSpec::Vec(inputs),
			Self::Hash => PipelinePortSpec::Static(&[("data", PipelineDataType::Binary)]),
			Self::Print => PipelinePortSpec::VecOwned(vec![(
				"data".into(),
				PipelineDataType::Reference {
					class: ClassHandle::from(1),
				},
			)]),

			// Audio
			Self::ExtractTags { .. } => {
				PipelinePortSpec::Static(&[("data", PipelineDataType::Binary)])
			}
			Self::StripTags => PipelinePortSpec::Static(&[("data", PipelineDataType::Binary)]),
			Self::ExtractCovers => PipelinePortSpec::Static(&[("data", PipelineDataType::Binary)]),

			Self::File => PipelinePortSpec::Static(&[("path", PipelineDataType::Text)]),
			Self::Dataset { attrs, .. } => PipelinePortSpec::VecOwned(
				attrs
					.iter()
					.map(|(x, y)| (x.clone().into(), y.clone()))
					.collect(),
			),
		}
	}

	fn outputs(&self, ctx: Arc<<Self::NodeType as PipelineNode>::RunContext>) -> PipelinePortSpec {
		match self {
			// Magic
			Self::ConstantNode { value } => {
				PipelinePortSpec::VecOwned(vec![("out".into(), value.get_type())])
			}

			// Util
			Self::IfNone => PipelinePortSpec::Static(&[("out", PipelineDataType::Text)]),
			Self::Hash => PipelinePortSpec::Static(&[("hash", PipelineDataType::Text)]),
			Self::Print => PipelinePortSpec::Static(&[]),
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

			Self::File => PipelinePortSpec::Static(&[
				("file_name", PipelineDataType::Text),
				("data", PipelineDataType::Binary),
			]),

			// TODO: add output
			Self::Dataset { class, .. } => {
				let d = ctx.dataset.lock().unwrap();
				let class = d.get_class(class).unwrap().unwrap();
				PipelinePortSpec::VecOwned(vec![(
					"added_item".into(),
					PipelineDataType::Reference { class },
				)])
			}
		}
	}
}