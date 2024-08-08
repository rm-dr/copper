use serde::de::DeserializeOwned;
use std::fmt::Debug;

use crate::{errors::PipelineError, labels::PipelinePortLabel, NDataStub};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineNodeState {
	/// This node has not been started
	NotStarted,

	/// This node has more work to do
	Pending,

	/// This node has output all its data and should not be run again
	Done,
}

impl PipelineNodeState {
	/// Is this [`PipelineNodeState::NotStarted`]?
	pub fn is_notstarted(&self) -> bool {
		matches!(self, Self::NotStarted)
	}

	/// Is this [`PipelineNodeState::Pending`]?
	pub fn is_pending(&self) -> bool {
		matches!(self, Self::Pending)
	}

	/// Is this [`PipelineNodeState::Done`]?
	pub fn is_done(&self) -> bool {
		matches!(self, Self::Done)
	}
}

pub trait PipelineNode {
	/// Extra resources available to nodes
	type NodeContext: Send + Sync;

	/// The kind of data this node handles
	type DataType: PipelineData;

	/// Initialize this node.
	/// This is called only once, when this node's inputs are ready.
	///
	/// A node's state should be [`PipelineNodeState::NotStarted`] before `init()` is called,
	/// and [`PipelineNodeState::Pending`] or [`PipelineNodeState::Done`] afterwards.
	///
	/// Note that this method can send data. For nodes that do very little computation,
	/// `init()` might be the only method that does meaningful work. This should be rare,
	/// though: `init()` blocks the main thread, and should *never* take a long time to run.
	///
	/// - Usually, `init()` sets up a node's inputs and returns [`PipelineNodeState::Pending`].
	/// - An `init()` call that takes too long will slow down *all* pipelines.
	/// - If `init()` returns [`PipelineNodeState::Done`], `run()` is never called.
	fn init<F>(
		&mut self,
		ctx: &Self::NodeContext,
		input: Vec<Self::DataType>,

		// Call this when data is ready.
		// Arguments are (port idx, data).
		//
		// This must be called *exactly once* for each of this port's outputs,
		// across both `init()` and `run()`.
		// (not enforced, but the pipeline will panic or hang if this is violated.)
		// TODO: enforce
		send_data: F,
	) -> Result<PipelineNodeState, PipelineError>
	where
		F: Fn(usize, Self::DataType) -> Result<(), PipelineError>;

	/// Run this node.
	/// This is always run in a worker thread.
	/// All heavy computation goes here.
	fn run<F>(
		&mut self,
		_ctx: &Self::NodeContext,
		_send_data: F,
	) -> Result<PipelineNodeState, PipelineError>
	where
		F: Fn(usize, Self::DataType) -> Result<(), PipelineError>,
	{
		Ok(PipelineNodeState::Done)
	}
}

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

pub trait PipelineData
where
	Self: Debug + Clone + Send + Sync,
{
	type DataStub: PipelineDataStub;

	/// Transform this data container into its type.
	fn as_stub(&self) -> Self::DataStub;

	/// Create an "empty" node of the given type.
	fn new_empty(stub: Self::DataStub) -> Self;
}

pub trait PipelineDataStub
where
	Self: Debug + PartialEq + Eq + Clone + Copy + Send + Sync,
{
}
