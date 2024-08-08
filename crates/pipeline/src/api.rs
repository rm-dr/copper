use serde::de::DeserializeOwned;
use std::{fmt::Debug, sync::Arc};

use crate::{errors::PipelineError, portspec::PipelinePortSpec};

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

		ctx: Arc<Self::NodeContext>,

		// TODO: provide args one at a time
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
		_ctx: Arc<Self::NodeContext>,
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
	type NodeType: PipelineNode + Sync + Send + 'static;

	/// Turn this stub into a proper node instance.
	fn build(
		&self,
		ctx: Arc<<Self::NodeType as PipelineNode>::NodeContext>,
		name: &str,
	) -> Self::NodeType;

	/// Return the inputs the node generated from this stub will take
	fn inputs(
		&self,
		ctx: Arc<<Self::NodeType as PipelineNode>::NodeContext>,
	) -> PipelinePortSpec<<<Self::NodeType as PipelineNode>::DataType as PipelineData>::DataStub>;

	/// Return the outputs the node generated from this stub will produce
	fn outputs(
		&self,
		ctx: Arc<<Self::NodeType as PipelineNode>::NodeContext>,
	) -> PipelinePortSpec<<<Self::NodeType as PipelineNode>::DataType as PipelineData>::DataStub>;
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
