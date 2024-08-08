use std::fmt::Debug;
use ufo_pipeline::{
	api::{PipelineNode, PipelineNodeState},
	errors::PipelineError,
	labels::PipelineNodeLabel,
};
use ufo_storage::data::StorageData;

use crate::{input::file::FileInput, output::storage::StorageOutput, UFOContext};

use super::{
	nodetype::UFONodeType,
	tags::{extractcovers::ExtractCovers, extracttags::ExtractTags, striptags::StripTags},
	util::{constant::Constant, hash::Hash, ifnone::IfNone, noop::Noop, print::Print},
};

pub enum UFONodeInstance {
	// Utility nodes
	Constant {
		node_type: UFONodeType,
		node: Constant,
	},
	IfNone {
		node_type: UFONodeType,
		name: PipelineNodeLabel,
		node: IfNone,
	},
	Noop {
		node_type: UFONodeType,
		name: PipelineNodeLabel,
		node: Noop,
	},
	Print {
		node_type: UFONodeType,
		name: PipelineNodeLabel,
		node: Print,
	},
	Hash {
		node_type: UFONodeType,
		name: PipelineNodeLabel,
		node: Hash,
	},

	// Audio nodes
	ExtractTags {
		node_type: UFONodeType,
		name: PipelineNodeLabel,
		node: ExtractTags,
	},
	StripTags {
		node_type: UFONodeType,
		name: PipelineNodeLabel,
		node: StripTags,
	},
	ExtractCovers {
		node_type: UFONodeType,
		name: PipelineNodeLabel,
		node: ExtractCovers,
	},

	File {
		node_type: UFONodeType,
		name: PipelineNodeLabel,
		node: FileInput,
	},

	Dataset {
		node_type: UFONodeType,
		name: PipelineNodeLabel,
		node: StorageOutput,
	},
}

impl Debug for UFONodeInstance {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Constant { .. } => write!(f, "ConstantNode"),
			Self::ExtractTags { name, .. } => write!(f, "ExtractTags({name})"),
			Self::IfNone { name, .. } => write!(f, "IfNone({name})"),
			Self::Noop { name, .. } => write!(f, "Noop({name})"),
			Self::Print { name, .. } => write!(f, "Print({name})"),
			Self::Hash { name, .. } => write!(f, "Hash({name})"),
			Self::StripTags { name, .. } => write!(f, "StripTags({name})"),
			Self::ExtractCovers { name, .. } => write!(f, "ExtractCovers({name})"),
			Self::Dataset { name, .. } => write!(f, "Dataset({name})"),
			Self::File { name, .. } => write!(f, "File({name})"),
		}
	}
}

impl PipelineNode for UFONodeInstance {
	type NodeContext = UFOContext;
	type DataType = StorageData;

	fn init<F>(
		&mut self,
		ctx: &Self::NodeContext,
		input: Vec<Self::DataType>,
		send_data: F,
	) -> Result<PipelineNodeState, PipelineError>
	where
		F: Fn(usize, Self::DataType) -> Result<(), PipelineError>,
	{
		match self {
			// Utility
			Self::Constant { node, .. } => node.init(ctx, input, send_data),
			Self::IfNone { node, .. } => node.init(ctx, input, send_data),
			Self::Noop { node, .. } => node.init(ctx, input, send_data),
			Self::Print { node, .. } => node.init(ctx, input, send_data),
			Self::Hash { node, .. } => node.init(ctx, input, send_data),

			// Audio
			Self::ExtractTags { node, .. } => node.init(ctx, input, send_data),
			Self::StripTags { node, .. } => node.init(ctx, input, send_data),
			Self::ExtractCovers { node, .. } => node.init(ctx, input, send_data),

			Self::Dataset { node, .. } => node.init(ctx, input, send_data),
			Self::File { node, .. } => node.init(ctx, input, send_data),
		}
	}

	fn run<F>(
		&mut self,
		ctx: &Self::NodeContext,
		send_data: F,
	) -> Result<PipelineNodeState, PipelineError>
	where
		F: Fn(usize, Self::DataType) -> Result<(), PipelineError>,
	{
		match self {
			Self::Dataset { node, .. } => node.run(ctx, send_data),
			Self::File { node, .. } => node.run(ctx, send_data),

			// Utility
			Self::Constant { node, .. } => node.run(ctx, send_data),
			Self::IfNone { node, .. } => node.run(ctx, send_data),
			Self::Noop { node, .. } => node.run(ctx, send_data),
			Self::Print { node, .. } => node.run(ctx, send_data),
			Self::Hash { node, .. } => node.run(ctx, send_data),

			// Audio
			Self::ExtractTags { node, .. } => node.run(ctx, send_data),
			Self::StripTags { node, .. } => node.run(ctx, send_data),
			Self::ExtractCovers { node, .. } => node.run(ctx, send_data),
		}
	}
}

impl UFONodeInstance {
	pub fn get_type(&self) -> &UFONodeType {
		match self {
			| Self::Dataset { node_type, .. }
			| Self::File { node_type, .. }

			// Utility
			| Self::IfNone { node_type, .. }
			| Self::Noop { node_type, .. }
			| Self::Hash { node_type, .. }
			| Self::Print { node_type, .. }
			| Self::Constant { node_type, .. }

			// Audio
			| Self::ExtractTags { node_type, .. }
			| Self::StripTags { node_type, .. }
			| Self::ExtractCovers { node_type, .. } => node_type,
		}
	}
}
