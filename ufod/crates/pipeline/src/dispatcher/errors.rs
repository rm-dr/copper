use std::{error::Error, fmt::Display};

/// An error we encounter when trying to register a node
#[derive(Debug)]
pub enum RegisterNodeError {
	/// We tried to register a node with a type string that is already used
	AlreadyExists,
}

impl Display for RegisterNodeError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::AlreadyExists => write!(f, "A node with this name already exists"),
		}
	}
}

impl Error for RegisterNodeError {}
