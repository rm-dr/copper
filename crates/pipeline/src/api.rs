//! Traits that allow external code to defune pipeline nodes
use serde::de::DeserializeOwned;
use std::{error::Error, fmt::Debug};

use crate::{labels::PipelinePortLabel, NDataStub};

/// The state of a [`PipelineNode`] at a point in time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineNodeState {
	/// This node has more work to do
	/// and is waiting to be `run()`.
	///
	/// This status always comes with a message, telling us
	/// why this node isn't `Done` yet.
	Pending(&'static str),

	/// This node has output all its data
	/// and will not be run again.
	Done,
}

impl PipelineNodeState {
	/// Is this [`PipelineNodeState::Pending`]?
	pub fn is_pending(&self) -> bool {
		matches!(self, Self::Pending(_))
	}

	/// Is this [`PipelineNodeState::Done`]?
	pub fn is_done(&self) -> bool {
		matches!(self, Self::Done)
	}
}

/// An instance of a pipeline node, with some state.
///
/// When a pipeline is run, a [`PipelineNode`] is created for each of its nodes.
///
/// A [`PipelineNode`] is used to run exactly one pipeline instance,
/// and is dropped when that pipeline finishes.
pub trait PipelineNode {
	/// Extra resources available when building nodes
	type NodeContext: Send + Sync;

	/// The kind of data this node handles
	type DataType: PipelineData;

	/// The kind of error this node can produce when running
	type ErrorType: Error + Send + Sync;

	/// If true, run this node in the main loop instead of starting a thread.
	///
	/// This should be `true` for nodes that do very little computation, and
	/// `false` for everything else.
	fn quick_run(&self) -> bool {
		false
	}

	/// Collect inputs queued for this node.
	/// Called before each call to `run()``.
	fn take_input<F>(
		&mut self,
		// (target port, data)
		input: (usize, Self::DataType),
		send_data: F,
	) -> Result<(), Self::ErrorType>
	where
		F: Fn(usize, Self::DataType) -> Result<(), Self::ErrorType>;

	/// Run this node.
	/// This is always run in a worker thread.
	fn run<F>(&mut self, _send_data: F) -> Result<PipelineNodeState, Self::ErrorType>
	where
		F: Fn(usize, Self::DataType) -> Result<(), Self::ErrorType>,
	{
		Ok(PipelineNodeState::Done)
	}
}

/// An object that represents a "type" of pipeline node.
/// Stubs are small and stateless.
pub trait PipelineNodeStub
where
	Self: Debug + Clone + DeserializeOwned + Sync + Send,
{
	/// The type of node this stub produces
	type NodeType: PipelineNode + Send + 'static;

	/// Turn this stub into a proper node instance.
	fn build(
		&self,
		ctx: &<Self::NodeType as PipelineNode>::NodeContext,
		name: &str,
	) -> Self::NodeType;

	/// How many inputs does this node produce?
	fn n_inputs(&self, ctx: &<Self::NodeType as PipelineNode>::NodeContext) -> usize;

	/// Find the index of the input with the given name.
	/// Returns `None` if no such input exists.
	fn input_with_name(
		&self,
		ctx: &<Self::NodeType as PipelineNode>::NodeContext,
		input_name: &PipelinePortLabel,
	) -> Option<usize>;

	/// The default input type for each port.
	/// `input_compatible_with` should return `true` for each of these types.
	fn input_default_type(
		&self,
		ctx: &<Self::NodeType as PipelineNode>::NodeContext,
		input_idx: usize,
	) -> NDataStub<Self::NodeType>;

	/// Can the specified inport port consume the given data type?
	/// This allows inputs to consume many types of data.
	fn input_compatible_with(
		&self,
		ctx: &<Self::NodeType as PipelineNode>::NodeContext,
		input_idx: usize,
		input_type: NDataStub<Self::NodeType>,
	) -> bool;

	/// How many inputs does this node produce?
	fn n_outputs(&self, ctx: &<Self::NodeType as PipelineNode>::NodeContext) -> usize;

	/// Find the index of the output with the given name.
	/// Returns `None` if no such output exists.
	fn output_with_name(
		&self,
		ctx: &<Self::NodeType as PipelineNode>::NodeContext,
		output_name: &PipelinePortLabel,
	) -> Option<usize>;

	/// What type of data does the output with the given index produce?
	fn output_type(
		&self,
		ctx: &<Self::NodeType as PipelineNode>::NodeContext,
		output_idx: usize,
	) -> NDataStub<Self::NodeType>;
}

/// An immutable bit of data inside a pipeline.
///
/// These should be easy to clone. [`PipelineData`]s that
/// carry something big probably wrap it in an [`std::sync::Arc`].
pub trait PipelineData
where
	Self: Debug + Clone + Send + Sync,
{
	/// The stub type that represents this node.
	type DataStub: PipelineDataStub;

	/// Transform this data container into its type.
	fn as_stub(&self) -> Self::DataStub;

	/// Create an "empty" node of the given type.
	fn new_empty(stub: Self::DataStub) -> Self;
}

/// A "type" of [`PipelineData`].
///
/// This does NOT carry data. Rather, it tells us
/// what *kind* of data a pipeline inputs/outputs.
pub trait PipelineDataStub
where
	Self: Debug + PartialEq + Eq + Clone + Copy + Send + Sync,
{
}
