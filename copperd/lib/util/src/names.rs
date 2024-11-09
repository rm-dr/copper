//! Object name utilities
use thiserror::Error;

/// The ways a name may be invalid
#[derive(Debug, Error)]
pub enum NameError {
	/// This name is empty
	#[error("name cannot be empty")]
	Empty,

	/// This name is entirely whitespace
	#[error("name cannot be entirely whitespace")]
	IsWhitespace,

	/// This name has leading or trailing whitespace
	#[error("name cannot have leading or trailing whitespace")]
	TrimWhitespace,
}

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
