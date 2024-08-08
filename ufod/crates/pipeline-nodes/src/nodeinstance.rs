use std::fmt::Debug;
use ufo_pipeline::{
	api::{PipelineNode, PipelineNodeState},
	labels::PipelineNodeID,
};

use crate::{
	data::UFOData,
	database::{additem::AddItem, finditem::FindItem},
	errors::PipelineError,
	input::file::FileReader,
	tags::{extractcovers::ExtractCovers, striptags::StripTags},
	util::hash::Hash,
	UFOContext,
};

use super::{
	nodetype::UFONodeType,
	tags::extracttags::ExtractTags,
	util::{constant::Constant, ifnone::IfNone, noop::Noop},
};

pub enum UFONodeInstance {
	// Utility nodes
	Constant {
		node_type: UFONodeType,
		node: Constant,
	},
	IfNone {
		node_type: UFONodeType,
		name: PipelineNodeID,
		node: IfNone,
	},
	Noop {
		node_type: UFONodeType,
		name: PipelineNodeID,
		node: Noop,
	},
	Hash {
		node_type: UFONodeType,
		name: PipelineNodeID,
		node: Hash,
	},

	// Audio nodes
	ExtractTags {
		node_type: UFONodeType,
		name: PipelineNodeID,
		node: ExtractTags,
	},
	StripTags {
		node_type: UFONodeType,
		name: PipelineNodeID,
		node: StripTags,
	},
	ExtractCovers {
		node_type: UFONodeType,
		name: PipelineNodeID,
		node: ExtractCovers,
	},

	File {
		node_type: UFONodeType,
		name: PipelineNodeID,
		node: FileReader,
	},

	AddItem {
		node_type: UFONodeType,
		name: PipelineNodeID,
		node: AddItem,
	},
	FindItem {
		node_type: UFONodeType,
		name: PipelineNodeID,
		node: FindItem,
	},
}

impl Debug for UFONodeInstance {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Constant { .. } => write!(f, "ConstantNode"),
			Self::ExtractTags { name, .. } => write!(f, "ExtractTags({name})"),
			Self::IfNone { name, .. } => write!(f, "IfNone({name})"),
			Self::Noop { name, .. } => write!(f, "Noop({name})"),
			Self::Hash { name, .. } => write!(f, "Hash({name})"),
			Self::StripTags { name, .. } => write!(f, "StripTags({name})"),
			Self::ExtractCovers { name, .. } => write!(f, "ExtractCovers({name})"),
			Self::AddItem { name, .. } => write!(f, "AddItem({name})"),
			Self::File { name, .. } => write!(f, "File({name})"),
			Self::FindItem { name, .. } => write!(f, "FindItem({name})"),
		}
	}
}

impl PipelineNode for UFONodeInstance {
	type NodeContext = UFOContext;
	type DataType = UFOData;
	type ErrorType = PipelineError;

	fn quick_run(&self) -> bool {
		match self {
			Self::AddItem { node, .. } => node.quick_run(),
			Self::FindItem { node, .. } => node.quick_run(),
			Self::File { node, .. } => node.quick_run(),

			// Utility
			Self::Constant { node, .. } => node.quick_run(),
			Self::IfNone { node, .. } => node.quick_run(),
			Self::Noop { node, .. } => node.quick_run(),
			Self::Hash { node, .. } => node.quick_run(),

			// Audio
			Self::ExtractTags { node, .. } => node.quick_run(),
			Self::StripTags { node, .. } => node.quick_run(),
			Self::ExtractCovers { node, .. } => node.quick_run(),
		}
	}

	fn take_input(&mut self, portdata: (usize, UFOData)) -> Result<(), PipelineError> {
		match self {
			Self::AddItem { node, .. } => node.take_input(portdata),
			Self::FindItem { node, .. } => node.take_input(portdata),
			Self::File { node, .. } => node.take_input(portdata),

			// Utility
			Self::Constant { node, .. } => node.take_input(portdata),
			Self::IfNone { node, .. } => node.take_input(portdata),
			Self::Noop { node, .. } => node.take_input(portdata),
			Self::Hash { node, .. } => node.take_input(portdata),

			// Audio
			Self::ExtractTags { node, .. } => node.take_input(portdata),
			Self::StripTags { node, .. } => node.take_input(portdata),
			Self::ExtractCovers { node, .. } => node.take_input(portdata),
		}
	}

	fn run<F>(&mut self, send_data: F) -> Result<PipelineNodeState, PipelineError>
	where
		F: Fn(usize, Self::DataType) -> Result<(), PipelineError>,
	{
		match self {
			Self::AddItem { node, .. } => node.run(send_data),
			Self::FindItem { node, .. } => node.run(send_data),
			Self::File { node, .. } => node.run(send_data),

			// Utility
			Self::Constant { node, .. } => node.run(send_data),
			Self::IfNone { node, .. } => node.run(send_data),
			Self::Noop { node, .. } => node.run(send_data),
			Self::Hash { node, .. } => node.run(send_data),

			// Audio
			Self::ExtractTags { node, .. } => node.run(send_data),
			Self::StripTags { node, .. } => node.run(send_data),
			Self::ExtractCovers { node, .. } => node.run(send_data),
		}
	}
}

impl UFONodeInstance {
	pub fn get_type(&self) -> &UFONodeType {
		match self {
			| Self::AddItem { node_type, .. }
			| Self::FindItem { node_type, .. }
			| Self::File { node_type, .. }

			// Utility
			| Self::IfNone { node_type, .. }
			| Self::Noop { node_type, .. }
			| Self::Hash { node_type, .. }
			| Self::Constant { node_type, .. }

			// Audio
			| Self::ExtractTags { node_type, .. }
			| Self::StripTags { node_type, .. }
			| Self::ExtractCovers { node_type, .. } => node_type,
		}
	}
}