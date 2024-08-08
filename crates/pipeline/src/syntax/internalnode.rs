use serde::{de::DeserializeOwned, Deserialize};
use std::fmt::Debug;

use crate::{
	api::{PipelineNode, PipelineNodeStub},
	labels::PipelineLabel,
	portspec::PipelinePortSpec,
	NDataStub,
};

#[derive(Clone, Deserialize)]
#[serde(tag = "type")]
#[serde(deny_unknown_fields)]
#[serde(bound = "StubType: DeserializeOwned")]
pub(crate) enum InternalNodeStub<StubType: PipelineNodeStub> {
	Pipeline {
		pipeline: PipelineLabel,
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
		ctx: &<Self::NodeType as PipelineNode>::NodeContext,
		name: &str,
	) -> Self::NodeType {
		match self {
			Self::Pipeline { .. } => unreachable!(),
			Self::User(n) => StubType::build(n, ctx, name),
		}
	}

	fn input_compatible_with(
		&self,
		ctx: &<Self::NodeType as PipelineNode>::NodeContext,
		input_idx: usize,
		input_type: NDataStub<Self::NodeType>,
	) -> bool {
		match self {
			Self::Pipeline { .. } => unreachable!(),
			Self::User(n) => n.input_compatible_with(ctx, input_idx, input_type),
		}
	}

	fn inputs(
		&self,
		ctx: &<Self::NodeType as PipelineNode>::NodeContext,
	) -> PipelinePortSpec<NDataStub<Self::NodeType>> {
		match self {
			Self::Pipeline { .. } => unreachable!(),
			Self::User(n) => n.inputs(ctx),
		}
	}

	fn outputs(
		&self,
		ctx: &<Self::NodeType as PipelineNode>::NodeContext,
	) -> PipelinePortSpec<NDataStub<Self::NodeType>> {
		match self {
			Self::Pipeline { .. } => unreachable!(),
			Self::User(n) => n.outputs(ctx),
		}
	}
}
