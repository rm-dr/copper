use smartstring::{LazyCompact, SmartString};
use std::{error::Error, fmt::Display, sync::Arc};
use tokio::sync::oneshot;

use super::PortName;

/// An error we encounter while running a node
#[derive(Debug, Clone)]
pub enum RunNodeError {
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

	/// A generic I/O error
	IoError(Arc<std::io::Error>),

	/// We encountered a RecvError while awaiting
	/// an input to this node
	InputReceiveError(oneshot::error::RecvError),

	/// An arbitrary error
	Other(Arc<dyn Error + Sync + Send + 'static>),
}

impl Display for RunNodeError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::IoError(_) => write!(f, "I/O error"),
			Self::MissingInput { port } => write!(f, "we did not receive input on port `{port}`"),
			Self::Other(_) => write!(f, "Generic error"),
			Self::InputReceiveError(_) => write!(f, "error while receiving input"),

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

impl Error for RunNodeError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::Other(x) => Some(x.as_ref()),
			Self::InputReceiveError(x) => Some(x),
			_ => return None,
		}
	}
}

impl From<std::io::Error> for RunNodeError {
	fn from(value: std::io::Error) -> Self {
		Self::IoError(Arc::new(value))
	}
}

impl From<oneshot::error::RecvError> for RunNodeError {
	fn from(value: oneshot::error::RecvError) -> Self {
		Self::InputReceiveError(value)
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
