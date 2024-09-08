//! Object name utilities

use std::{error::Error, fmt::Display};

/// The ways a name may be invalid
#[derive(Debug)]
pub enum NameError {
	/// This name is empty
	Empty,

	/// This name is entirely whitespace
	IsWhitespace,

	/// This name has leading or trailing whitespace
	TrimWhitespace,
}

impl Display for NameError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Empty => write!(f, "name cannot be empty"),
			Self::IsWhitespace => write!(f, "name cannot be entirely whitespace"),
			Self::TrimWhitespace => write!(f, "name cannot have leading or trailing whitespace"),
		}
	}
}

impl Error for NameError {}

/// Check the given name for errors.
pub fn check_name(name: &str) -> Result<(), NameError> {
	if name.is_empty() {
		return Err(NameError::Empty);
	}

	let trimmed = name.trim();
	if trimmed.is_empty() {
		return Err(NameError::IsWhitespace);
	}

	if trimmed.len() != name.len() {
		return Err(NameError::TrimWhitespace);
	}

	return Ok(());
}
