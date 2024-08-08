use crossbeam::channel::Receiver;
use serde::{de::DeserializeOwned, Deserialize};
use std::fmt::Debug;

use crate::{
	api::{PipelineNode, PipelineNodeStub},
	labels::PipelineLabel,
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

		input_receiver: Receiver<(
			// The port this data goes to
			usize,
			// The data
			<Self::NodeType as PipelineNode>::DataType,
		)>,
	) -> Self::NodeType {
		match self {
			Self::Pipeline { .. } => unreachable!(),
			Self::User(n) => n.build(ctx, name, input_receiver),
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

	fn n_inputs(&self, ctx: &<Self::NodeType as PipelineNode>::NodeContext) -> usize {
		match self {
			Self::Pipeline { .. } => unreachable!(),
			Self::User(n) => n.n_inputs(ctx),
		}
	}

	fn input_default_type(
		&self,
		ctx: &<Self::NodeType as PipelineNode>::NodeContext,
		input_idx: usize,
	) -> NDataStub<Self::NodeType> {
		match self {
			Self::Pipeline { .. } => unreachable!(),
			Self::User(n) => n.input_default_type(ctx, input_idx),
		}
	}

	fn input_with_name(
		&self,
		ctx: &<Self::NodeType as PipelineNode>::NodeContext,
		input_name: &crate::labels::PipelinePortLabel,
	) -> Option<usize> {
		match self {
			Self::Pipeline { .. } => unreachable!(),
			Self::User(n) => n.input_with_name(ctx, input_name),
		}
	}

	fn n_outputs(&self, ctx: &<Self::NodeType as PipelineNode>::NodeContext) -> usize {
		match self {
			Self::Pipeline { .. } => unreachable!(),
			Self::User(n) => n.n_outputs(ctx),
		}
	}

	fn output_with_name(
		&self,
		ctx: &<Self::NodeType as PipelineNode>::NodeContext,
		output_name: &crate::labels::PipelinePortLabel,
	) -> Option<usize> {
		match self {
			Self::Pipeline { .. } => unreachable!(),
			Self::User(n) => n.output_with_name(ctx, output_name),
		}
	}

	fn output_type(
		&self,
		ctx: &<Self::NodeType as PipelineNode>::NodeContext,
		output_idx: usize,
	) -> NDataStub<Self::NodeType> {
		match self {
			Self::Pipeline { .. } => unreachable!(),
			Self::User(n) => n.output_type(ctx, output_idx),
		}
	}
}
