use serde::Deserialize;
use serde_with::serde_as;
use ufo_audiofile::common::tagtype::TagType;
use ufo_metadb::data::{HashType, MetaDbDataStub};
use ufo_pipeline::{
	api::{PipelineNode, PipelineNodeStub},
	labels::PipelinePortLabel,
	NDataStub,
};

use super::{
	nodeinstance::UFONodeInstance,
	tags::extracttags::ExtractTags,
	util::{constant::Constant, ifnone::IfNone, noop::Noop, print::Print},
};
use crate::{
	data::UFOData,
	database::{additem::AddItem, finditem::FindItem},
	input::file::FileReader,
	tags::{extractcovers::ExtractCovers, striptags::StripTags},
	traits::UFONode,
	util::hash::Hash,
};

#[serde_as]
#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
#[serde(deny_unknown_fields)]
pub enum UFONodeType {
	// Utility nodes
	Print,
	Constant {
		value: UFOData,
	},
	IfNone {
		data_type: MetaDbDataStub,
	},
	Hash {
		hash_type: HashType,
	},
	Noop {
		#[serde(rename = "input")]
		#[serde_as(as = "serde_with::Map<_, _>")]
		inputs: Vec<(PipelinePortLabel, MetaDbDataStub)>,
	},

	// Audio nodes
	ExtractCovers,
	StripTags,
	ExtractTags {
		tags: Vec<TagType>,
	},

	// Etc
	File,
	AddItem {
		class: String,
	},
	FindItem {
		class: String,
		by_attr: String,
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
				node: Constant::new(ctx, value.clone()),
			},

			// Util
			UFONodeType::IfNone { .. } => UFONodeInstance::IfNone {
				node_type: self.clone(),
				name: name.into(),
				node: IfNone::new(ctx),
			},
			UFONodeType::Noop { inputs } => UFONodeInstance::Noop {
				node_type: self.clone(),
				name: name.into(),
				node: Noop::new(ctx, inputs.clone()),
			},
			UFONodeType::Print => UFONodeInstance::Print {
				node_type: self.clone(),
				name: name.into(),
				node: Print::new(ctx),
			},

			UFONodeType::Hash { hash_type } => UFONodeInstance::Hash {
				node_type: self.clone(),
				name: name.into(),
				node: Hash::new(ctx, *hash_type),
			},

			// Audio
			UFONodeType::StripTags => UFONodeInstance::StripTags {
				node_type: self.clone(),
				name: name.into(),
				node: StripTags::new(ctx),
			},
			UFONodeType::ExtractTags { tags } => UFONodeInstance::ExtractTags {
				node_type: self.clone(),
				name: name.into(),
				node: ExtractTags::new(ctx, tags.clone()),
			},
			UFONodeType::ExtractCovers => UFONodeInstance::ExtractCovers {
				node_type: self.clone(),
				name: name.into(),
				node: ExtractCovers::new(ctx),
			},
			UFONodeType::File => UFONodeInstance::File {
				node_type: self.clone(),
				name: name.into(),
				node: FileReader::new(ctx),
			},
			UFONodeType::AddItem { class } => {
				let mut d = ctx.database.lock().unwrap();
				let class = d.get_class(class).unwrap().unwrap();
				let attrs = d.class_get_attrs(class).unwrap();

				UFONodeInstance::AddItem {
					node_type: self.clone(),
					name: name.into(),
					node: AddItem::new(ctx, class, attrs),
				}
			}

			UFONodeType::FindItem { class, by_attr } => {
				let mut d = ctx.database.lock().unwrap();
				// TODO: handle errors
				let class = d.get_class(class).unwrap().unwrap();
				let attrs = d.get_attr(class, &by_attr).unwrap().unwrap();
				drop(d);

				UFONodeInstance::FindItem {
					node_type: self.clone(),
					name: name.into(),
					node: FindItem::new(ctx, class, attrs).unwrap(),
				}
			}
		}
	}

	fn n_inputs(&self, ctx: &<Self::NodeType as PipelineNode>::NodeContext) -> usize {
		match self {
			Self::Constant { .. } => Constant::n_inputs(self, ctx),
			Self::IfNone { .. } => IfNone::n_inputs(self, ctx),
			Self::Print => Print::n_inputs(self, ctx),
			Self::Hash { .. } => Hash::n_inputs(self, ctx),
			Self::Noop { .. } => Noop::n_inputs(self, ctx),
			Self::ExtractCovers => ExtractCovers::n_inputs(self, ctx),
			Self::StripTags => StripTags::n_inputs(self, ctx),
			Self::ExtractTags { .. } => ExtractTags::n_inputs(self, ctx),
			Self::File => FileReader::n_inputs(self, ctx),
			Self::AddItem { .. } => AddItem::n_inputs(self, ctx),
			Self::FindItem { .. } => FindItem::n_inputs(self, ctx),
		}
	}

	fn input_compatible_with(
		&self,
		ctx: &<Self::NodeType as PipelineNode>::NodeContext,
		input_idx: usize,
		input_type: NDataStub<Self::NodeType>,
	) -> bool {
		match self {
			Self::Constant { .. } => {
				Constant::input_compatible_with(self, ctx, input_idx, input_type)
			}
			Self::IfNone { .. } => IfNone::input_compatible_with(self, ctx, input_idx, input_type),
			Self::Print => Print::input_compatible_with(self, ctx, input_idx, input_type),
			Self::Hash { .. } => Hash::input_compatible_with(self, ctx, input_idx, input_type),
			Self::Noop { .. } => Noop::input_compatible_with(self, ctx, input_idx, input_type),
			Self::ExtractCovers => {
				ExtractCovers::input_compatible_with(self, ctx, input_idx, input_type)
			}
			Self::StripTags => StripTags::input_compatible_with(self, ctx, input_idx, input_type),
			Self::ExtractTags { .. } => {
				ExtractTags::input_compatible_with(self, ctx, input_idx, input_type)
			}
			Self::File => FileReader::input_compatible_with(self, ctx, input_idx, input_type),
			Self::AddItem { .. } => {
				AddItem::input_compatible_with(self, ctx, input_idx, input_type)
			}
			Self::FindItem { .. } => {
				FindItem::input_compatible_with(self, ctx, input_idx, input_type)
			}
		}
	}

	fn input_default_type(
		&self,
		ctx: &<Self::NodeType as PipelineNode>::NodeContext,
		input_idx: usize,
	) -> NDataStub<Self::NodeType> {
		match self {
			Self::Constant { .. } => Constant::input_default_type(self, ctx, input_idx),
			Self::IfNone { .. } => IfNone::input_default_type(self, ctx, input_idx),
			Self::Print => Print::input_default_type(self, ctx, input_idx),
			Self::Hash { .. } => Hash::input_default_type(self, ctx, input_idx),
			Self::Noop { .. } => Noop::input_default_type(self, ctx, input_idx),
			Self::ExtractCovers => ExtractCovers::input_default_type(self, ctx, input_idx),
			Self::StripTags => StripTags::input_default_type(self, ctx, input_idx),
			Self::ExtractTags { .. } => ExtractTags::input_default_type(self, ctx, input_idx),
			Self::File => FileReader::input_default_type(self, ctx, input_idx),
			Self::AddItem { .. } => AddItem::input_default_type(self, ctx, input_idx),
			Self::FindItem { .. } => FindItem::input_default_type(self, ctx, input_idx),
		}
	}

	fn input_with_name(
		&self,
		ctx: &<Self::NodeType as PipelineNode>::NodeContext,
		input_name: &PipelinePortLabel,
	) -> Option<usize> {
		match self {
			Self::Constant { .. } => Constant::input_with_name(self, ctx, input_name),
			Self::IfNone { .. } => IfNone::input_with_name(self, ctx, input_name),
			Self::Print => Print::input_with_name(self, ctx, input_name),
			Self::Hash { .. } => Hash::input_with_name(self, ctx, input_name),
			Self::Noop { .. } => Noop::input_with_name(self, ctx, input_name),
			Self::ExtractCovers => ExtractCovers::input_with_name(self, ctx, input_name),
			Self::StripTags => StripTags::input_with_name(self, ctx, input_name),
			Self::ExtractTags { .. } => ExtractTags::input_with_name(self, ctx, input_name),
			Self::File => FileReader::input_with_name(self, ctx, input_name),
			Self::AddItem { .. } => AddItem::input_with_name(self, ctx, input_name),
			Self::FindItem { .. } => FindItem::input_with_name(self, ctx, input_name),
		}
	}

	fn n_outputs(&self, ctx: &<Self::NodeType as PipelineNode>::NodeContext) -> usize {
		match self {
			Self::Constant { .. } => Constant::n_outputs(self, ctx),
			Self::IfNone { .. } => IfNone::n_outputs(self, ctx),
			Self::Print => Print::n_outputs(self, ctx),
			Self::Hash { .. } => Hash::n_outputs(self, ctx),
			Self::Noop { .. } => Noop::n_outputs(self, ctx),
			Self::ExtractCovers => ExtractCovers::n_outputs(self, ctx),
			Self::StripTags => StripTags::n_outputs(self, ctx),
			Self::ExtractTags { .. } => ExtractTags::n_outputs(self, ctx),
			Self::File => FileReader::n_outputs(self, ctx),
			Self::AddItem { .. } => AddItem::n_outputs(self, ctx),
			Self::FindItem { .. } => FindItem::n_outputs(self, ctx),
		}
	}

	fn output_type(
		&self,
		ctx: &<Self::NodeType as PipelineNode>::NodeContext,
		output_idx: usize,
	) -> NDataStub<Self::NodeType> {
		match self {
			Self::Constant { .. } => Constant::output_type(self, ctx, output_idx),
			Self::IfNone { .. } => IfNone::output_type(self, ctx, output_idx),
			Self::Print => Print::output_type(self, ctx, output_idx),
			Self::Hash { .. } => Hash::output_type(self, ctx, output_idx),
			Self::Noop { .. } => Noop::output_type(self, ctx, output_idx),
			Self::ExtractCovers => ExtractCovers::output_type(self, ctx, output_idx),
			Self::StripTags => StripTags::output_type(self, ctx, output_idx),
			Self::ExtractTags { .. } => ExtractTags::output_type(self, ctx, output_idx),
			Self::File => FileReader::output_type(self, ctx, output_idx),
			Self::AddItem { .. } => AddItem::output_type(self, ctx, output_idx),
			Self::FindItem { .. } => FindItem::output_type(self, ctx, output_idx),
		}
	}

	fn output_with_name(
		&self,
		ctx: &<Self::NodeType as PipelineNode>::NodeContext,
		output_name: &PipelinePortLabel,
	) -> Option<usize> {
		match self {
			Self::Constant { .. } => Constant::output_with_name(self, ctx, output_name),
			Self::IfNone { .. } => IfNone::output_with_name(self, ctx, output_name),
			Self::Print => Print::output_with_name(self, ctx, output_name),
			Self::Hash { .. } => Hash::output_with_name(self, ctx, output_name),
			Self::Noop { .. } => Noop::output_with_name(self, ctx, output_name),
			Self::ExtractCovers => ExtractCovers::output_with_name(self, ctx, output_name),
			Self::StripTags => StripTags::output_with_name(self, ctx, output_name),
			Self::ExtractTags { .. } => ExtractTags::output_with_name(self, ctx, output_name),
			Self::File => FileReader::output_with_name(self, ctx, output_name),
			Self::AddItem { .. } => AddItem::output_with_name(self, ctx, output_name),
			Self::FindItem { .. } => FindItem::output_with_name(self, ctx, output_name),
		}
	}
}
