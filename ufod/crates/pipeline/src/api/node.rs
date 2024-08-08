use super::{PipelineData, RunNodeError};
use crate::labels::PipelinePortID;

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

/// Information about a node. Depends on a node's parameters.
/// Used to validate connections.
pub trait NodeInfo<DataType: PipelineData> {
	/// Get this pipeline's inputs
	fn inputs(&self) -> &[(PipelinePortID, DataType::DataStubType)];

	/// Get this pipeline's outputs
	fn outputs(&self) -> &[(PipelinePortID, DataType::DataStubType)];
}

/// An instance of a pipeline node, with some state.
///
/// When a pipeline is run, a [`PipelineNode`] is created for each of its nodes.
///
/// A [`PipelineNode`] is used to run exactly one pipeline instance,
/// and is dropped when that pipeline finishes.
pub trait PipelineNode<DataType: PipelineData>: Sync + Send {
	/// If true, run this node in the main loop instead of starting a thread.
	///
	/// This should be `true` for nodes that do no heavy computation, and
	/// `false` for everything else. If this is true, `run` will block the
	/// async event loop, and thus cannot take a long time to run.
	fn quick_run(&self) -> bool {
		false
	}

	/// Accept input data to a port of this node.
	fn take_input(&mut self, target_port: usize, input_data: DataType) -> Result<(), RunNodeError>;

	/// Run this node.
	/// This is always run in a worker thread.
	fn run(
		&mut self,
		send_data: &dyn Fn(usize, DataType) -> Result<(), RunNodeError>,
	) -> Result<PipelineNodeState, RunNodeError>;

	/// Get this node's info
	fn get_info(&self) -> &dyn NodeInfo<DataType>;
}
