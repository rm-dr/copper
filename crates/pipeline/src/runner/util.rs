use crate::{data::PipelineData, node::PipelineNodeState};

#[derive(Debug)]
pub(super) enum EdgeValue {
	/// This edge is waiting on another node to run
	Uninitialized,

	/// This edge has data that is ready to be used
	/// (Only valid for Edge::PortToPort)
	Data(PipelineData),

	/// This edge had data, but it has been consumed
	/// (Only valid for Edge::PortToPort)
	Consumed,

	/// This edge's source node has finised running
	/// (Only valid for Edge::After)
	AfterReady,
}

impl EdgeValue {
	pub fn unwrap(self) -> PipelineData {
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
	Running,
	NotRunning(PipelineNodeState),
}

impl NodeRunState {
	pub fn is_running(&self) -> bool {
		matches!(self, Self::Running)
	}

	pub fn is_done(&self) -> bool {
		matches!(self, Self::NotRunning(PipelineNodeState::Done))
	}

	pub fn is_notstarted(&self) -> bool {
		matches!(self, Self::NotRunning(PipelineNodeState::NotStarted))
	}

	pub fn is_pending(&self) -> bool {
		matches!(self, Self::NotRunning(PipelineNodeState::Pending))
	}
}
