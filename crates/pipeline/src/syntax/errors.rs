//! Error helpers for pipeline spec parsing

use std::{error::Error, fmt::Display};
use ufo_util::data::PipelineDataType;

use super::{
	labels::{PipelineNodeLabel, PipelinePortLabel},
	ports::{NodeInput, NodeOutput},
};

/// A node specification in a [`PipelinePrepareError`]
#[derive(Debug)]
pub enum PipelineErrorNode {
	/// The pipeline's output node
	PipelineOutput,
	PipelineInput,

	/// A named node created by the user
	Named(PipelineNodeLabel),
}

/// An error we encounter when a pipeline spec is invalid
#[derive(Debug)]
pub enum PipelinePrepareError {
	/// We could not open a pipeline spec file
	CouldNotOpenFile {
		/// The error we encountered
		error: std::io::Error,
	},

	/// We could not read a pipeline spec file
	CouldNotReadFile {
		/// The error we encountered
		error: std::io::Error,
	},

	/// We could not parse a pipeline spec file
	CouldNotParseFile {
		/// The error we encountered
		error: toml::de::Error,
	},

	/// There is no node named `node` in this pipeline.
	NoNode {
		/// The node label that doesn't exist
		node: PipelineNodeLabel,
		/// We tried to connect to `node` from this input.
		caused_by: NodeInput,
	},

	/// `node` has no input named `input`.
	/// This is triggered when we specify an input that doesn't exist.
	NoNodeInput {
		/// The node we tried to reference
		node: PipelineErrorNode,
		/// The input name that doesn't exist
		input: PipelinePortLabel,
	},

	/// `node` has no output named `output`.
	NoNodeOutput {
		/// The node we tried to connect to
		node: PipelineErrorNode,
		/// The output name that doesn't exist
		output: PipelinePortLabel,
		/// The node input we tried to connect to `output`
		caused_by: NodeInput,
	},

	/// We tried to connect `input` to `output`,
	/// but their types don't match.
	TypeMismatch {
		/// The output we tried to connect
		output: NodeOutput,
		/// The input we tried to connect
		input: NodeInput,
	},

	/// We tried to connect an inline type to `input`,
	/// but their types don't match.
	InlineTypeMismatch {
		/// The type our inline data has
		inline_type: PipelineDataType,
		/// The input we tried to connect it to
		input: NodeInput,
	},

	/// This graph has a cycle containing `node`
	HasCycle,
}

impl Display for PipelinePrepareError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::CouldNotOpenFile { .. } => {
				writeln!(f, "PipelinePrepareError: Could not open file")
			}
			Self::CouldNotReadFile { .. } => {
				writeln!(f, "PipelinePrepareError: Could not read file")
			}
			Self::CouldNotParseFile { .. } => {
				writeln!(f, "PipelinePrepareError: Could not parse file")
			}
			Self::NoNode { node, caused_by } => {
				writeln!(
					f,
					"PipelinePrepareError: No such node `{node:?}`. Caused by `{caused_by:?}`."
				)
			}
			Self::NoNodeInput { node, input } => {
				writeln!(
					f,
					"PipelinePrepareError: Node `{node:?}` has no input `{input}`"
				)
			}
			Self::NoNodeOutput {
				node,
				output,
				caused_by,
			} => {
				writeln!(
					f,
					"PipelinePrepareError: Node `{node:?}` has no output `{output}`. Caused by `{caused_by:?}`."
				)
			}
			Self::TypeMismatch { output, input } => {
				writeln!(
					f,
					"PipelinePrepareError: `{output:?}` and `{input:?}` have different types."
				)
			}
			Self::InlineTypeMismatch { input, inline_type } => {
				writeln!(
					f,
					"PipelinePrepareError: `{input:?}` cannot consume inline type `{inline_type}`"
				)
			}
			Self::HasCycle => {
				writeln!(f, "PipelinePrepareError: This pipeline has a cycle.")
			}
		}
	}
}

impl Error for PipelinePrepareError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::CouldNotOpenFile { error } => Some(error),
			Self::CouldNotReadFile { error } => Some(error),
			Self::CouldNotParseFile { error } => Some(error),
			_ => None,
		}
	}
}
