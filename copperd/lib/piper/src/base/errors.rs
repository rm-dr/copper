use smartstring::{LazyCompact, SmartString};
use std::{error::Error, sync::Arc};
use thiserror::Error;
use tokio::task::JoinError;

use super::{NodeId, PortName};

/// An error we encounter while running a node
#[derive(Debug, Clone, Error)]
pub enum RunNodeError {
	//
	// MARK: Errors in pipeline definition
	//
	//
	//
	#[error("database error")]
	DbError(#[from] Arc<sqlx::Error>),

	/// We expected a parameter, but it wasn't there
	#[error("Unexpected parameter `{parameter}`")]
	UnexpectedParameter { parameter: SmartString<LazyCompact> },

	/// A parameter had an unexpected type
	#[error("Bad type for parameter `{parameter}`")]
	BadParameterType { parameter: SmartString<LazyCompact> },

	/// We expected a parameter, but it wasn't there
	#[error("Missing parameter `{parameter}`")]
	MissingParameter { parameter: SmartString<LazyCompact> },

	/// Generic parameter error
	#[error("Bad parameter `{parameter}`: {message}")]
	BadParameterOther {
		parameter: SmartString<LazyCompact>,
		message: String,
	},

	/// We did not receive a required input
	#[error("we did not receive input on port `{port}`")]
	MissingInput { port: PortName },

	/// A required input was connected, but received null data
	#[error("received null data on required port `{port}`")]
	RequiredInputNull { port: PortName },

	/// We received an input on a port we don't recognize
	#[error("received input on unrecognized port `{port}`")]
	UnrecognizedInput { port: PortName },

	/// We received data with an invalid type on the given port
	#[error("received bad data type on port `{port}`")]
	BadInputType { port: PortName },

	/// An edge was connected to an output port of a node that doesn't exist
	#[error("edge connected to an unrecognized output port `{port}`")]
	UnrecognizedOutput { port: PortName },

	/// We tried to take an action we are not authorized to take
	/// (e.g, we tried to run `AddItem` on another user's dataset)
	#[error("Not authorized: {message}")]
	NotAuthorized { message: String },

	//
	// MARK: Node runtime errors
	//
	//
	//
	/// A generic I/O error
	#[error("i/o error")]
	IoError(#[from] Arc<std::io::Error>),

	/// An arbitrary error
	#[error("generic error")]
	Other(#[from] Arc<dyn Error + Sync + Send + 'static>),

	/// A node task threw a JoinError
	#[error("error while joining task")]
	NodeTaskJoinError(#[from] Arc<JoinError>),

	/// One output port got input twice
	#[error("node {node_id} ({node_type}) sent data to output {port} twice.")]
	OutputPortSetTwice {
		node_id: NodeId,
		node_type: SmartString<LazyCompact>,
		port: PortName,
	},
}

impl From<std::io::Error> for RunNodeError {
	fn from(value: std::io::Error) -> Self {
		Self::IoError(Arc::new(value))
	}
}

impl From<JoinError> for RunNodeError {
	fn from(value: JoinError) -> Self {
		Self::NodeTaskJoinError(Arc::new(value))
	}
}

/// An error we encounter while running a node
#[derive(Debug, Error)]
pub enum ProcessSignalError {
	/// We tried to process data we don't know how to handle
	/// (e.g, we tried to process binary data with a format we don't support)
	///
	/// Comes with a helpful message
	#[error("Unsupported format: {0}")]
	UnsupportedFormat(String),

	/// We tried to connect to an input port that doesn't exist,
	/// or we received data on a port that doesn't exist
	#[error("tried to connect an input port that doesn't exist")]
	InputPortDoesntExist,

	/// We received input with an invalid data type
	#[error("received input with invalid data type")]
	InputWithBadType,

	/// A required input did not receive data before being disconnected
	#[error("a required input did not receive data")]
	RequiredInputEmpty,
}
