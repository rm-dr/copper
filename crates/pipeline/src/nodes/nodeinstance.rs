use smartstring::{LazyCompact, SmartString};
use std::{fmt::Debug, sync::Arc};
use ufo_util::data::{PipelineData, PipelineDataType};

use super::{ifnone::IfNone, nodetype::PipelineNodeType, tags::ExtractTags};
use crate::{errors::PipelineError, portspec::PipelinePortSpec, PipelineStatelessRunner};

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
	fn run(&self, data: Vec<Arc<PipelineData>>) -> Result<Vec<Arc<PipelineData>>, PipelineError> {
		match self {
			Self::ExternalNode => Ok(Default::default()),
			Self::ConstantNode(x) => Ok(vec![x.clone()]),
			Self::ExtractTags { node, .. } => node.run(data),
			Self::IfNone { node, .. } => node.run(data),
		}
	}
}

impl PipelineNodeInstance {
	pub fn inputs(&self) -> Option<PipelinePortSpec> {
		match self {
			Self::ExternalNode => None,
			Self::ConstantNode(_) => None,
			Self::ExtractTags { .. } => Some(PipelinePortSpec::Static(&[
				("title", PipelineDataType::Text),
				("album", PipelineDataType::Text),
				("artist", PipelineDataType::Text),
				("genre", PipelineDataType::Text),
				("comment", PipelineDataType::Text),
				("track", PipelineDataType::Text),
				("disk", PipelineDataType::Text),
				("disk_total", PipelineDataType::Text),
				("year", PipelineDataType::Text),
			])),
			Self::IfNone { .. } => Some(PipelineNodeType::IfNone.inputs()),
		}
	}
}
