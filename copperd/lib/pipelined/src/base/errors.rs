use smartstring::{LazyCompact, SmartString};
use std::{error::Error, fmt::Display, sync::Arc};
use tokio::{sync::mpsc, task::JoinError};

use super::{NodeId, NodeOutput, PipelineData, PortName};

/// An error we encounter while running a node
#[derive(Debug, Clone)]
pub enum RunNodeError<DataType: PipelineData> {
	//
	// MARK: Errors in pipeline definition
	//
	//
	//
	/// We expected a parameter, but it wasn't there
	UnexpectedParameter { parameter: SmartString<LazyCompact> },

	/// A parameter had an unexpected type
	BadParameterType { parameter: SmartString<LazyCompact> },

	/// We expected a parameter, but it wasn't there
	MissingParameter { parameter: SmartString<LazyCompact> },

	/// Generic parameter error
	BadParameterOther {
		parameter: SmartString<LazyCompact>,
		message: String,
	},

	/// We did not receive a required input
	MissingInput { port: PortName },

	/// A required input was connected, but received null data
	RequiredInputNull { port: PortName },

	/// We received an input on a port we don't recognize
	UnrecognizedInput { port: PortName },

	/// We received data with an invalid type on the given port
	BadInputType { port: PortName },

	/// An edge was connected to an output port of a node that doesn't exist
	UnrecognizedOutput { port: PortName },

	//
	// MARK: Node runtime errors
	//
	//
	//
	/// A generic I/O error
	IoError(Arc<std::io::Error>),

	/// We tried to read from a byte stream, but that stream overflowed
	/// and we missed data. If this happens, either a node isn't reading
	/// stream data fast enough, or our max buffer size is too small.
	StreamReceiverLagged,

	/// An arbitrary error
	Other(Arc<dyn Error + Sync + Send + 'static>),

	//
	// MARK: Critical errors
	// (If we encounter these, our code is wrong)
	//
	//
	//
	/// We encountered a SendError while sending node output
	OutputSendError(mpsc::error::SendError<NodeOutput<DataType>>),

	/// A node task threw a JoinError
	NodeTaskJoinError(Arc<JoinError>),

	/// One output port got input twice
	OutputPortSetTwice {
		node_id: NodeId,
		node_type: SmartString<LazyCompact>,
		port: PortName,
	},
}

impl<DataType: PipelineData> Display for RunNodeError<DataType> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::IoError(_) => write!(f, "I/O error"),
			Self::MissingInput { port } => write!(f, "we did not receive input on port `{port}`"),
			Self::Other(_) => write!(f, "Generic error"),
			Self::OutputPortSetTwice {
				node_id,
				node_type,
				port,
			} => write!(
				f,
				"node {node_id} ({node_type}) sent data to output {port} twice."
			),
			Self::OutputSendError(_) => write!(f, "error while sending output"),
			Self::NodeTaskJoinError(_) => write!(f, "error while joining task"),
			Self::StreamReceiverLagged => write!(f, "stream receiver lagged"),

			Self::BadInputType { port } => {
				write!(f, "received bad data type on port `{port}`")
			}

			Self::RequiredInputNull { port } => {
				write!(f, "received null data on required port `{port}`")
			}

			Self::BadParameterOther { message, parameter } => {
				write!(f, "Bad parameter `{parameter}`: {message}")
			}

			Self::BadParameterType { parameter } => {
				write!(f, "Bad type for parameter `{parameter}`")
			}

			Self::MissingParameter { parameter } => {
				write!(f, "Missing parameter `{parameter}`")
			}

			Self::UnexpectedParameter { parameter } => {
				write!(f, "Unexpected parameter `{parameter}`")
			}

			Self::UnrecognizedInput { port } => {
				write!(f, "received input on unrecognized port `{port}`")
			}

			Self::UnrecognizedOutput { port } => {
				write!(f, "edge connected to an unrecognized output port `{port}`")
			}
		}
	}
}

impl<DataType: PipelineData> Error for RunNodeError<DataType> {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::Other(x) => Some(x.as_ref()),
			Self::OutputSendError(x) => Some(x),
			Self::NodeTaskJoinError(x) => Some(x),
			_ => return None,
		}
	}
}

impl<DataType: PipelineData> From<std::io::Error> for RunNodeError<DataType> {
	fn from(value: std::io::Error) -> Self {
		Self::IoError(Arc::new(value))
	}
}

impl<DataType: PipelineData> From<mpsc::error::SendError<NodeOutput<DataType>>>
	for RunNodeError<DataType>
{
	fn from(value: mpsc::error::SendError<NodeOutput<DataType>>) -> Self {
		Self::OutputSendError(value)
	}
}

impl<DataType: PipelineData> From<JoinError> for RunNodeError<DataType> {
	fn from(value: JoinError) -> Self {
		Self::NodeTaskJoinError(Arc::new(value))
	}
}

/// An error we encounter while running a node
#[derive(Debug)]
pub enum ProcessSignalError {
	/// We tried to process data we don't know how to handle
	/// (e.g, we tried to process binary data with a format we don't support)
	///
	/// Comes with a helpful message
	UnsupportedFormat(String),

	/// We tried to connect to an input port that doesn't exist,
	/// or we received data on a port that doesn't exist
	InputPortDoesntExist,

	/// We received input with an invalid data type
	InputWithBadType,

	/// A required input did not receive data before being disconnected
	RequiredInputEmpty,
}

impl Display for ProcessSignalError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::RequiredInputEmpty => write!(f, "a required input did not receive data"),
			Self::InputWithBadType => write!(f, "received input with invalid data type"),
			Self::UnsupportedFormat(msg) => write!(f, "Unsupported format: {msg}"),
			Self::InputPortDoesntExist => {
				write!(f, "tried to connect an input port that doesn't exist")
			}
		}
	}
}

impl Error for ProcessSignalError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			_ => return None,
		}
	}
}
