//! Object name utilities

use std::{error::Error, fmt::Display};

/// The ways a name may be invalid
#[derive(Debug)]
pub enum NameError {
	/// This name is either empty, or entirely whitespace.
	Empty,
}

impl Display for NameError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Empty => write!(f, "name cannot be empty"),
		}
	}
}

impl Error for NameError {}

/// Check the given object name for errors and perform minor cleanup
/// (whitespace trimming, etc)
///
/// If it has errors, return `Err(_)`.
/// If it has is ok, return the name we should use.
pub fn clean_name(name: &str) -> Result<String, NameError> {
	let name = name.trim();
	if name.is_empty() {
		return Err(NameError::Empty);
	}

	return Ok(name.into());
}
