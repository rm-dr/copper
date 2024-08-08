// Utilities
pub mod nodeinstance;
pub mod nodetype;

// Node implementations
pub mod tags;
pub mod util;

use crate::{data::PipelineData, errors::PipelineError};

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
	/// Initialize this node.
	/// This is called only once, when this node's inputs are ready.
	///
	/// A node's state should be [`PipelineNodeState::NotStarted`] before `init()` is called,
	/// and [`PipelineNodeState::NotStarted`] or [`PipelineNodeState::Done`] afterwards.
	///
	/// Note that this method can send data. For nodes that do very little computation,
	/// `init()` might be the only method that does meaningful work. This should be rare,
	/// though: `init()` blocks the main thread, and should *never* take a long time to run.
	///
	/// If `init()` returns [`PipelineNodeState::Done`], `run()` is never called.
	fn init<F>(
		&mut self,
		// Call this when data is ready.
		// Arguments are (port idx, data).
		//
		// This must be called *exactly once* for each of this port's outputs,
		// across both `init()` and `run()`.
		// (not enforced, but the pipeline will panic or hang if this is violated.)
		// TODO: enforce
		send_data: F,

		// TODO: provide args one at a time
		input: Vec<PipelineData>,
	) -> Result<PipelineNodeState, PipelineError>
	where
		F: Fn(usize, PipelineData) -> Result<(), PipelineError>;

	/// Run this node.
	/// This is always run in a worker thread. All heavy computation goes here.
	fn run<F>(&mut self, _send_data: F) -> Result<PipelineNodeState, PipelineError>
	where
		F: Fn(usize, PipelineData) -> Result<(), PipelineError>,
	{
		Ok(PipelineNodeState::Done)
	}
}
