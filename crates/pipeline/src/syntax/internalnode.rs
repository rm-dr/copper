use serde::{de::DeserializeOwned, Deserialize};
use std::{fmt::Debug, sync::Arc};

use crate::{
	api::{PipelineData, PipelineNode, PipelineNodeStub},
	portspec::PipelinePortSpec,
};

#[derive(Clone, Deserialize)]
#[serde(tag = "type")]
#[serde(deny_unknown_fields)]
#[serde(bound = "StubType: DeserializeOwned")]
pub(crate) enum InternalNodeStub<StubType: PipelineNodeStub> {
	Pipeline {
		pipeline: String,
	},

	#[serde(untagged)]
	User(StubType),
}

impl<StubType: PipelineNodeStub> Debug for InternalNodeStub<StubType> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Pipeline { .. } => todo!(),
			Self::User(x) => x.fmt(f),
		}
	}
}

impl<StubType: PipelineNodeStub> PipelineNodeStub for InternalNodeStub<StubType> {
	type NodeType = StubType::NodeType;

	fn build(
		&self,
		ctx: Arc<<Self::NodeType as PipelineNode>::NodeContext>,
		name: &str,
	) -> Self::NodeType {
		match self {
			Self::Pipeline { .. } => unreachable!(),
			Self::User(n) => StubType::build(n, ctx, name),
		}
	}

	fn inputs(
		&self,
		ctx: Arc<<Self::NodeType as PipelineNode>::NodeContext>,
	) -> PipelinePortSpec<<<Self::NodeType as PipelineNode>::DataType as PipelineData>::DataStub> {
		match self {
			Self::Pipeline { .. } => unreachable!(),
			Self::User(n) => n.inputs(ctx),
		}
	}

	fn outputs(
		&self,
		ctx: Arc<<Self::NodeType as PipelineNode>::NodeContext>,
	) -> PipelinePortSpec<<<Self::NodeType as PipelineNode>::DataType as PipelineData>::DataStub> {
		match self {
			Self::Pipeline { .. } => unreachable!(),
			Self::User(n) => n.outputs(ctx),
		}
	}
}
