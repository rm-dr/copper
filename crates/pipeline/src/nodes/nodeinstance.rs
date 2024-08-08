use smartstring::{LazyCompact, SmartString};
use std::fmt::Debug;

use super::{
	nodetype::PipelineNodeType,
	tags::{extractcovers::ExtractCovers, extracttags::ExtractTags, striptags::StripTags},
	util::{constant::Constant, hash::Hash, ifnone::IfNone, noop::Noop},
	PipelineNode,
};
use crate::{data::PipelineData, errors::PipelineError};

pub enum PipelineNodeInstance {
	// Each node instance must have a node_type field,
	// which is guaranteed to have the correct enum variant.
	// (see PipelineNodeType::build())

	// Magic nodes
	PipelineInputs {
		node_type: PipelineNodeType,
	},
	PipelineOutputs {
		node_type: PipelineNodeType,
	},

	// Utility nodes
	Constant {
		node_type: PipelineNodeType,
		node: Constant,
	},
	IfNone {
		node_type: PipelineNodeType,
		name: SmartString<LazyCompact>,
		node: IfNone,
	},
	Noop {
		node_type: PipelineNodeType,
		name: SmartString<LazyCompact>,
		node: Noop,
	},
	Hash {
		node_type: PipelineNodeType,
		name: SmartString<LazyCompact>,
		node: Hash,
	},

	// Audio nodes
	ExtractTags {
		node_type: PipelineNodeType,
		name: SmartString<LazyCompact>,
		node: ExtractTags,
	},
	StripTags {
		node_type: PipelineNodeType,
		name: SmartString<LazyCompact>,
		node: StripTags,
	},
	ExtractCovers {
		node_type: PipelineNodeType,
		name: SmartString<LazyCompact>,
		node: ExtractCovers,
	},
}

impl Debug for PipelineNodeInstance {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::PipelineInputs { .. } => write!(f, "PipelineInputs"),
			Self::PipelineOutputs { .. } => write!(f, "PipelineOutputs"),
			Self::Constant { .. } => write!(f, "ConstantNode"),
			Self::ExtractTags { name, .. } => write!(f, "ExtractTags({name})"),
			Self::IfNone { name, .. } => write!(f, "IfNone({name})"),
			Self::Noop { name, .. } => write!(f, "Noop({name})"),
			Self::Hash { name, .. } => write!(f, "Hash({name})"),
			Self::StripTags { name, .. } => write!(f, "StripTags({name})"),
			Self::ExtractCovers { name, .. } => write!(f, "ExtractCovers({name})"),
		}
	}
}

impl PipelineNode for PipelineNodeInstance {
	fn init<F>(
		&mut self,
		// Call this when data is ready.
		// Arguments are (port idx, data).
		//
		// This must be called *exactly once* for each of this port's outputs,
		// across both `init()` and `run()`.
		// (not enforced, but the pipeline will panic or hang if this is violated.)
		// TODO: enforce
		send_data: F,

		input: Vec<PipelineData>,
	) -> Result<super::PipelineNodeState, PipelineError>
	where
		F: Fn(usize, PipelineData) -> Result<(), PipelineError>,
	{
		match self {
			// These are handled as special cases by Pipeline::run().
			Self::PipelineInputs { .. } => unreachable!(),
			Self::PipelineOutputs { .. } => unreachable!(),

			// Utility
			Self::Constant { node, .. } => node.init(send_data, input),
			Self::IfNone { node, .. } => node.init(send_data, input),
			Self::Noop { node, .. } => node.init(send_data, input),
			Self::Hash { node, .. } => node.init(send_data, input),

			// Audio
			Self::ExtractTags { node, .. } => node.init(send_data, input),
			Self::StripTags { node, .. } => node.init(send_data, input),
			Self::ExtractCovers { node, .. } => node.init(send_data, input),
		}
	}

	fn run<F>(&mut self, send_data: F) -> Result<super::PipelineNodeState, PipelineError>
	where
		F: Fn(usize, PipelineData) -> Result<(), PipelineError>,
	{
		match self {
			// These are handled as special cases by Pipeline::run().
			Self::PipelineInputs { .. } => unreachable!(),
			Self::PipelineOutputs { .. } => unreachable!(),

			// Nodes that are run here
			Self::Constant { node, .. } => node.run(send_data),

			// Utility
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

impl PipelineNodeInstance {
	pub fn get_type(&self) -> &PipelineNodeType {
		match self {
			// Magic
			Self::PipelineInputs { node_type, .. }
			| Self::PipelineOutputs { node_type, .. }
			| Self::Constant { node_type, .. }

			// Utility
			| Self::IfNone { node_type, .. }
			| Self::Noop { node_type, .. }
			| Self::Hash { node_type, .. }

			// Audio
			| Self::ExtractTags { node_type, .. }
			| Self::StripTags { node_type, .. }
			| Self::ExtractCovers { node_type, .. } => node_type,
		}
	}
}
