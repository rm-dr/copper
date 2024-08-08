//! Pipeline runner utilities

use crate::api::{PipelineData, PipelineNodeState};

#[derive(Debug)]
pub(super) enum EdgeValue<DataType: PipelineData> {
	/// This edge is waiting on another node to run
	Uninitialized,

	/// This edge has data that is ready to be used
	/// (Only valid for Edge::PortToPort)
	Data(DataType),

	/// This edge had data, but it has been consumed
	/// (Only valid for Edge::PortToPort)
	Consumed,

	/// This edge's source node has finised running
	/// (Only valid for Edge::After)
	AfterReady,
}

impl<DataType: PipelineData> EdgeValue<DataType> {
	/// Get the data inside an [`EdgeValue::Data`] or return `None`.
	pub fn unwrap(self) -> DataType {
		match self {
			Self::Data(x) => x,
			_ => panic!("tried to unwrap a non-Data Edgevalue"),
		}
	}
}

/// A wrapper around [`PipelineNodeState`] that keeps
/// track if a certain node is running *right now*.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum NodeRunState {
	/// This node is currently running in a thread.
	Running,

	/// This node is not actively running
	NotRunning(PipelineNodeState),
}

impl NodeRunState {
	/// Is this node [`NodeRunState::Running`]?
	pub fn is_running(&self) -> bool {
		matches!(self, Self::Running)
	}

	/// Is this node `NodeRunState::NotRunning(PipelineNodestate::Done)`?
	pub fn is_done(&self) -> bool {
		matches!(self, Self::NotRunning(PipelineNodeState::Done))
	}

	/// Is this node `NodeRunState::NotRunning(PipelineNodestate::NotStarted)`?
	pub fn is_notstarted(&self) -> bool {
		matches!(self, Self::NotRunning(PipelineNodeState::NotStarted))
	}

	/// Is this node `NodeRunState::NotRunning(PipelineNodestate::Pending)`?
	pub fn is_pending(&self) -> bool {
		matches!(self, Self::NotRunning(PipelineNodeState::Pending))
	}
}
