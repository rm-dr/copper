use serde::Deserialize;
use serde_with::serde_as;
use smartstring::{LazyCompact, SmartString};
use ufo_audiofile::common::tagtype::TagType;
use ufo_pipeline::{
	api::{PipelineData, PipelineNode, PipelineNodeStub},
	labels::PipelinePortLabel,
	portspec::PipelinePortSpec,
};
use ufo_storage::{
	api::ClassHandle,
	data::{HashType, StorageData, StorageDataStub},
};

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
	Constant {
		value: StorageData,
	},

	// Utility nodes
	IfNone,
	Hash,
	Print,
	Noop {
		#[serde(rename = "input")]
		#[serde_as(as = "serde_with::Map<_, _>")]
		inputs: Vec<(PipelinePortLabel, StorageDataStub)>,
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
		attrs: Vec<(SmartString<LazyCompact>, StorageDataStub)>,
	},
}

impl PipelineNodeStub for UFONodeType {
	type NodeType = UFONodeInstance;

	fn build(
		&self,
		ctx: &<Self::NodeType as PipelineNode>::NodeContext,
		name: &str,
	) -> UFONodeInstance {
		match self {
			// Magic
			UFONodeType::Constant { value } => UFONodeInstance::Constant {
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
				let mut d = ctx.dataset.lock().unwrap();
				let class = d.get_class(class).unwrap().unwrap();

				UFONodeInstance::Dataset {
					node_type: self.clone(),
					name: name.into(),
					node: StorageOutput::new(class.clone(), attrs.clone()),
				}
			}
		}
	}

	fn inputs(
		&self,
		_ctx: &<Self::NodeType as PipelineNode>::NodeContext,
	) -> PipelinePortSpec<<<Self::NodeType as PipelineNode>::DataType as PipelineData>::DataStub> {
		match self {
			// Util
			Self::Constant { .. } => PipelinePortSpec::Static(&[]),
			Self::IfNone => PipelinePortSpec::Static(&[
				("data", StorageDataStub::Text),
				("ifnone", StorageDataStub::Text),
			]),
			Self::Noop { inputs } => PipelinePortSpec::Vec(inputs),
			Self::Hash => PipelinePortSpec::Static(&[("data", StorageDataStub::Binary)]),
			Self::Print => PipelinePortSpec::VecOwned(vec![(
				"data".into(),
				StorageDataStub::Reference {
					// TODO: specify type as arg
					class: ClassHandle::from(2),
				},
			)]),

			// Audio
			Self::ExtractTags { .. } => {
				PipelinePortSpec::Static(&[("data", StorageDataStub::Binary)])
			}
			Self::StripTags => PipelinePortSpec::Static(&[("data", StorageDataStub::Binary)]),
			Self::ExtractCovers => PipelinePortSpec::Static(&[("data", StorageDataStub::Binary)]),

			Self::File => PipelinePortSpec::Static(&[("path", StorageDataStub::Path)]),
			Self::Dataset { attrs, .. } => PipelinePortSpec::VecOwned(
				attrs
					.iter()
					.map(|(x, y)| (x.clone().into(), y.clone()))
					.collect(),
			),
		}
	}

	fn outputs(
		&self,
		ctx: &<Self::NodeType as PipelineNode>::NodeContext,
	) -> PipelinePortSpec<<<Self::NodeType as PipelineNode>::DataType as PipelineData>::DataStub> {
		match self {
			// Magic
			Self::Constant { value } => {
				PipelinePortSpec::VecOwned(vec![("value".into(), value.as_stub())])
			}

			// Util
			Self::IfNone => PipelinePortSpec::Static(&[("out", StorageDataStub::Text)]),
			Self::Hash => PipelinePortSpec::Static(&[(
				"hash",
				StorageDataStub::Hash {
					format: HashType::SHA256,
				},
			)]),
			Self::Print => PipelinePortSpec::Static(&[]),
			Self::Noop { inputs } => PipelinePortSpec::Vec(inputs),

			// Audio
			Self::ExtractTags { tags } => PipelinePortSpec::VecOwned(
				tags.iter()
					.map(|x| (Into::<&str>::into(x).into(), StorageDataStub::Text))
					.collect(),
			),
			Self::StripTags => PipelinePortSpec::Static(&[("out", StorageDataStub::Binary)]),
			Self::ExtractCovers => {
				PipelinePortSpec::Static(&[("cover_data", StorageDataStub::Binary)])
			}

			Self::File => PipelinePortSpec::Static(&[
				("path", StorageDataStub::Path),
				("data", StorageDataStub::Binary),
			]),

			// TODO: add output
			Self::Dataset { class, .. } => {
				let mut d = ctx.dataset.lock().unwrap();
				let class = d.get_class(class).unwrap().unwrap();
				PipelinePortSpec::VecOwned(vec![(
					"added_item".into(),
					StorageDataStub::Reference { class },
				)])
			}
		}
	}
}
