use smartstring::{LazyCompact, SmartString};
use std::{fmt::Debug, sync::Arc};
use ufo_util::data::PipelineData;

use crate::{errors::PipelineError, PipelineStatelessRunner};

use super::{ifnone::IfNone, nodetype::PipelineNodeType, tags::ExtractTags};

pub enum PipelineNodeInstance {
	ExternalNode,
	ConstantNode(Arc<PipelineData>),
	ExtractTags {
		name: SmartString<LazyCompact>,
		node: ExtractTags,
	},
	IfNone {
		name: SmartString<LazyCompact>,
		node: IfNone,
	},
}

impl Debug for PipelineNodeInstance {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::ExternalNode => write!(f, "ExternalNode"),
			Self::ConstantNode(_) => write!(f, "ConstantNode"),
			Self::ExtractTags { name, .. } => write!(f, "ExtractTags({name})"),
			Self::IfNone { name, .. } => write!(f, "IfNone({name})"),
		}
	}
}

impl PipelineStatelessRunner for PipelineNodeInstance {
	fn run(
		&self,
		data_packet: Vec<Option<Arc<PipelineData>>>,
	) -> Result<Vec<Option<Arc<PipelineData>>>, PipelineError> {
		match self {
			Self::ExternalNode => Ok(Default::default()),
			Self::ConstantNode(x) => Ok(vec![Some(x.clone())]),
			Self::ExtractTags { node, .. } => node.run(data_packet),
			Self::IfNone { node, .. } => node.run(data_packet),
		}
	}
}

impl PipelineNodeInstance {
	pub fn get_type(&self) -> Option<PipelineNodeType> {
		match self {
			Self::ExternalNode => None,
			Self::ConstantNode(_) => None,
			Self::ExtractTags { .. } => Some(PipelineNodeType::ExtractTags),
			Self::IfNone { .. } => Some(PipelineNodeType::IfNone),
		}
	}
}
