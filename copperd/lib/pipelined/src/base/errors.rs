use smartstring::{LazyCompact, SmartString};
use std::{error::Error, fmt::Display};

/// An error we encounter when initializing a node
#[derive(Debug)]
pub enum InitNodeError {
	/// We got an unexpected number of parameters
	BadParameterCount {
		/// How many we expected
		expected: usize,
	},

	/// A parameter had an unexpected type
	BadParameterType {
		/// The parameter
		param_name: SmartString<LazyCompact>,
	},

	/// We expected a parameter, but it wasn't there
	MissingParameter {
		/// The parameter that was missing
		param_name: SmartString<LazyCompact>,
	},

	/// Generic parameter error
	BadParameterOther {
		/// The parameter that caused the error
		param_name: SmartString<LazyCompact>,

		/// A description of the error
		message: String,
	},

	/// An arbitrary error
	Other(Box<dyn Error + Sync + Send + 'static>),
}

impl Display for InitNodeError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Other(_) => write!(f, "Generic error"),
			Self::BadParameterOther {
				message,
				param_name,
			} => write!(f, "Bad parameter `{param_name}`: {message}"),
			Self::BadParameterCount { expected } => {
				write!(f, "Bad number of parameters: expected {expected}")
			}
			Self::BadParameterType { param_name } => {
				write!(f, "Bad type for parameter `{param_name}`")
			}
			Self::MissingParameter { param_name } => {
				write!(f, "Missing parameter `{param_name}`")
			}
		}
	}
}

impl Error for InitNodeError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::Other(x) => Some(x.as_ref()),
			_ => return None,
		}
	}
}

/// An error we encounter while running a node
#[derive(Debug)]
pub enum RunNodeError {
	/// A generic I/O error
	IoError(std::io::Error),

	/// An arbitrary error
	Other(Box<dyn Error + Sync + Send + 'static>),

	/// A required input was not connected before calling `run()`
	RequiredInputNotConnected,
}

impl Display for RunNodeError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::IoError(_) => write!(f, "I/O error"),
			Self::Other(_) => write!(f, "Generic error"),
			Self::RequiredInputNotConnected => write!(
				f,
				"a required input was not connected before running a node"
			),
		}
	}
}

impl Error for RunNodeError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::Other(x) => Some(x.as_ref()),
			_ => return None,
		}
	}
}

impl From<std::io::Error> for RunNodeError {
	fn from(value: std::io::Error) -> Self {
		Self::IoError(value)
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
