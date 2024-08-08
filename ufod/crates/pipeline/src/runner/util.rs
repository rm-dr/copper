//! Pipeline runner utilities

use crate::api::NodeState;

#[derive(Debug)]
pub(super) enum EdgeState {
	/// This is an `Edge::PortToPort`
	Data,

	// This is an Edge::After that is waiting for it's source node
	AfterWaiting,

	// This is an Edge::After whose source node has finished running.
	AfterReady,
}

/// A wrapper around [`PipelineNodeState`] that keeps
/// track if a certain node is running *right now*.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum NodeRunState {
	/// This node is currently queued in the thread pool.
	Running,

	/// This node is not actively running
	NotRunning(NodeState),
}

impl NodeRunState {
	/// Is this node [`NodeRunState::Running`]?
	pub fn is_running(&self) -> bool {
		matches!(self, Self::Running)
	}

	/// Is this node `NodeRunState::NotRunning(PipelineNodestate::Done)`?
	pub fn is_done(&self) -> bool {
		matches!(self, Self::NotRunning(NodeState::Done))
	}
}
