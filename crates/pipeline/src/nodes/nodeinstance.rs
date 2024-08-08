use smartstring::{LazyCompact, SmartString};
use std::{fmt::Debug, sync::Arc};
use ufo_util::data::PipelineData;

use super::{
	nodetype::PipelineNodeType,
	tags::{striptags::StripTags, tags::ExtractTags},
	util::ifnone::IfNone,
};
use crate::{errors::PipelineError, PipelineNode};

pub enum PipelineNodeInstance {
	// Each node instance must have a node_type field,
	// which is guaranteed to be correct by
	// PipelineNodeType::build().
	PipelineInputs {
		node_type: PipelineNodeType,
		input_values: Vec<Arc<PipelineData>>,
	},
	PipelineOutputs {
		node_type: PipelineNodeType,
	},
	ConstantNode {
		node_type: PipelineNodeType,
	},
	ExtractTags {
		node_type: PipelineNodeType,
		name: SmartString<LazyCompact>,
		node: ExtractTags,
	},
	IfNone {
		node_type: PipelineNodeType,
		name: SmartString<LazyCompact>,
		node: IfNone,
	},
	StripTags {
		node_type: PipelineNodeType,
		name: SmartString<LazyCompact>,
		node: StripTags,
	},
}

impl Debug for PipelineNodeInstance {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::PipelineInputs { .. } => write!(f, "PipelineInputs"),
			Self::PipelineOutputs { .. } => write!(f, "PipelineOutputs"),
			Self::ConstantNode { .. } => write!(f, "ConstantNode"),
			Self::ExtractTags { name, .. } => write!(f, "ExtractTags({name})"),
			Self::IfNone { name, .. } => write!(f, "IfNone({name})"),
			Self::StripTags { name, .. } => write!(f, "StripTags({name})"),
		}
	}
}

impl PipelineNode for PipelineNodeInstance {
	fn run<F>(&self, send_data: F, input: Vec<Arc<PipelineData>>) -> Result<(), PipelineError>
	where
		F: Fn(usize, Arc<PipelineData>) -> Result<(), PipelineError>,
	{
		match self {
			Self::PipelineInputs { input_values, .. } => {
				for (i, v) in input_values.iter().enumerate() {
					send_data(i, v.clone())?;
				}
				Ok(())
			}
			Self::PipelineOutputs { .. } => Ok(()),
			Self::ConstantNode { node_type } => match node_type {
				PipelineNodeType::ConstantNode { value } => {
					send_data(0, value.clone())?;
					Ok(())
				}
				_ => unreachable!(),
			},
			Self::ExtractTags { node, .. } => node.run(send_data, input),
			Self::IfNone { node, .. } => node.run(send_data, input),
			Self::StripTags { node, .. } => node.run(send_data, input),
		}
	}
}

impl PipelineNodeInstance {
	pub fn get_type(&self) -> &PipelineNodeType {
		match self {
			Self::PipelineInputs { node_type, .. }
			| Self::PipelineOutputs { node_type, .. }
			| Self::ConstantNode { node_type, .. }
			| Self::ExtractTags { node_type, .. }
			| Self::IfNone { node_type, .. }
			| Self::StripTags { node_type, .. } => &node_type,
		}
	}
}
