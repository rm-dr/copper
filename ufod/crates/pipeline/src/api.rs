//! Traits that allow external code to defune pipeline nodes
use serde::de::DeserializeOwned;
use std::{
	error::Error,
	fmt::{Debug, Display},
};

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

/// Information about an node's input port
pub struct NodeInputInfo<DataStubType> {
	/// The port's name
	pub name: PipelinePortID,

	/// The type of data this port accepts
	pub accepts_type: DataStubType,
}

/// Information about a node's output port
pub struct NodeOutputInfo<DataStubType> {
	/// This port's name
	pub name: PipelinePortID,

	/// The type of data this port produces
	pub produces_type: DataStubType,
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
	fn take_input(
		&mut self,
		target_port: usize,
		input_data: DataType,
	) -> Result<(), PipelineNodeError>;

	/// Run this node.
	/// This is always run in a worker thread.
	fn run(
		&mut self,
		send_data: &dyn Fn(usize, DataType) -> Result<(), PipelineNodeError>,
	) -> Result<PipelineNodeState, PipelineNodeError>;

	/// What inputs does this node take?
	fn inputs(&self) -> &[NodeInputInfo<DataType::DataStubType>];

	/// What outputs does this node produce?
	fn outputs(&self) -> &[NodeOutputInfo<DataType::DataStubType>];
}

/// An immutable bit of data inside a pipeline.
///
/// These should be easy to clone. [`PipelineData`]s that
/// carry something big should wrap it in an [`std::sync::Arc`].
///
/// The `Deserialize` implementation of this struct MUST NOT be transparent.
/// It should always be some sort of object. See the dispatcher param enums
/// for more details.
pub trait PipelineData
where
	Self: DeserializeOwned + Debug + Clone + Send + Sync + 'static,
{
	/// The stub type that represents this node.
	type DataStubType: PipelineDataStub;

	/// Transform this data container into its type.
	fn as_stub(&self) -> Self::DataStubType;

	/// Create an "empty" data of the given type.
	/// This is sent to all disconnected inputs.
	fn disconnected(stub: Self::DataStubType) -> Self;
}

/// A "type" of [`PipelineData`].
///
/// This does NOT carry data. Rather, it tells us
/// what *kind* of data a pipeline inputs/outputs.
///
/// The `Deserialize` implementation of this struct MUST NOT be transparent.
/// It should always be some sort of object. See the dispatcher param enums
/// for more details.
pub trait PipelineDataStub
where
	Self: DeserializeOwned + Debug + PartialEq + Eq + Clone + Copy + Send + Sync + 'static,
{
	/// If true, an input of type `superset` can accept data of type `self`.
	fn is_subset_of(&self, superset: &Self) -> bool;
}

/// Arbitrary additional information for a pipeline job.
pub trait PipelineJobContext
where
	Self: Send + Sync + 'static,
{
}
