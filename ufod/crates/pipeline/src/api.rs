//! Traits that allow external code to defune pipeline nodes
use serde::de::DeserializeOwned;
use std::{
	error::Error,
	fmt::{Debug, Display},
};

use crate::{labels::PipelinePortID, NDataStub};

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

/// An error a pipeline node can produce
#[derive(Debug)]
pub enum PipelineNodeError {
	/// A generic I/O error
	IoError(std::io::Error),

	/// We tried to process data we don't know how to handle
	/// (e.g, we tried to process binary data with a format we don't support)
	///
	/// Comes with a helpful message
	UnsupportedFormat(String),

	/// An arbitrary error
	Other(Box<dyn Error + Sync + Send>),
}

unsafe impl Send for PipelineNodeError {}
unsafe impl Sync for PipelineNodeError {}

impl Display for PipelineNodeError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::IoError(_) => write!(f, "I/O error"),
			Self::UnsupportedFormat(msg) => write!(f, "Unsupported format: {msg}"),
			Self::Other(_) => write!(f, "Generic error"),
		}
	}
}

impl Error for PipelineNodeError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::Other(x) => Some(x.as_ref()),
			_ => return None,
		}
	}
}

impl From<std::io::Error> for PipelineNodeError {
	fn from(value: std::io::Error) -> Self {
		PipelineNodeError::IoError(value)
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

	/// If true, run this node in the main loop instead of starting a thread.
	///
	/// This should be `true` for nodes that do very little computation, and
	/// `false` for everything else.
	fn quick_run(&self) -> bool {
		false
	}

	/// Collect inputs queued for this node.
	/// Called before each call to `run()``.
	fn take_input(
		&mut self,
		// (target port, data)
		input: (usize, Self::DataType),
	) -> Result<(), PipelineNodeError>;
	/// Run this node.
	/// This is always run in a worker thread.
	fn run<F>(&mut self, _send_data: F) -> Result<PipelineNodeState, PipelineNodeError>
	where
		F: Fn(usize, Self::DataType) -> Result<(), PipelineNodeError>,
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

	/// Errors we can encounter when getting node parameters
	type ErrorType: Error;

	/// Turn this stub into a proper node instance.
	fn build(
		&self,
		ctx: &<Self::NodeType as PipelineNode>::NodeContext,
		name: &str,
	) -> Result<Self::NodeType, Self::ErrorType>;

	/// How many inputs does this node produce?
	fn n_inputs(
		&self,
		ctx: &<Self::NodeType as PipelineNode>::NodeContext,
	) -> Result<usize, Self::ErrorType>;

	/// Find the index of the input with the given name.
	/// Returns `None` if no such input exists.
	fn input_with_name(
		&self,
		ctx: &<Self::NodeType as PipelineNode>::NodeContext,
		input_name: &PipelinePortID,
	) -> Result<Option<usize>, Self::ErrorType>;

	/// The default input type for each port.
	/// `input_compatible_with` should return `true` for each of these types.
	///
	/// This is used when we need a data stub for this input, but none is available.
	/// (for example, if we need to send `None` data to a disconnected input)
	fn input_default_type(
		&self,
		ctx: &<Self::NodeType as PipelineNode>::NodeContext,
		input_idx: usize,
	) -> Result<NDataStub<Self::NodeType>, Self::ErrorType>;

	/// Can the specified inport port consume the given data type?
	/// This allows inputs to consume many types of data.
	fn input_compatible_with(
		&self,
		ctx: &<Self::NodeType as PipelineNode>::NodeContext,
		input_idx: usize,
		input_type: NDataStub<Self::NodeType>,
	) -> Result<bool, Self::ErrorType>;

	/// How many inputs does this node produce?
	fn n_outputs(
		&self,
		ctx: &<Self::NodeType as PipelineNode>::NodeContext,
	) -> Result<usize, Self::ErrorType>;

	/// Find the index of the output with the given name.
	/// Returns `None` if no such output exists.
	fn output_with_name(
		&self,
		ctx: &<Self::NodeType as PipelineNode>::NodeContext,
		output_name: &PipelinePortID,
	) -> Result<Option<usize>, Self::ErrorType>;

	/// What type of data does the output with the given index produce?
	fn output_type(
		&self,
		ctx: &<Self::NodeType as PipelineNode>::NodeContext,
		output_idx: usize,
	) -> Result<NDataStub<Self::NodeType>, Self::ErrorType>;
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
	type DataStubType: PipelineDataStub;

	/// Transform this data container into its type.
	fn as_stub(&self) -> Self::DataStubType;

	/// Create an "empty" node of the given type.
	fn new_empty(stub: Self::DataStubType) -> Self;
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
