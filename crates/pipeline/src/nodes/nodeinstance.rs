use smartstring::{LazyCompact, SmartString};
use std::{fmt::Debug, sync::Arc};
use ufo_util::data::{PipelineData, PipelineDataType};

use super::{
	nodetype::PipelineNodeType,
	tags::{striptags::StripTags, tags::ExtractTags},
	util::ifnone::IfNone,
};
use crate::{errors::PipelineError, portspec::PipelinePortSpec, PipelineStatelessNode};

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
	StripTags {
		name: SmartString<LazyCompact>,
		node: StripTags,
	},
}

impl Debug for PipelineNodeInstance {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::ExternalNode => write!(f, "ExternalNode"),
			Self::ConstantNode(_) => write!(f, "ConstantNode"),
			Self::ExtractTags { name, .. } => write!(f, "ExtractTags({name})"),
			Self::IfNone { name, .. } => write!(f, "IfNone({name})"),
			Self::StripTags { name, .. } => write!(f, "StripTags({name})"),
		}
	}
}

impl PipelineStatelessNode for PipelineNodeInstance {
	fn run<F>(&self, send_data: F, input: Vec<Arc<PipelineData>>) -> Result<(), PipelineError>
	where
		F: Fn(usize, Arc<PipelineData>) -> Result<(), PipelineError>,
	{
		match self {
			Self::ExternalNode => Ok(()),
			Self::ConstantNode(x) => {
				send_data(0, x.clone())?;
				Ok(())
			}
			Self::ExtractTags { node, .. } => node.run(send_data, input),
			Self::IfNone { node, .. } => node.run(send_data, input),
			Self::StripTags { node, .. } => node.run(send_data, input),
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
			Self::StripTags { .. } => Some(PipelineNodeType::StripTags.inputs()),
		}
	}
}
