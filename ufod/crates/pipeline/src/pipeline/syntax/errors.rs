//! Errors we can encounter when parsing a pipeline

use std::{error::Error, fmt::Display};

use smartstring::{LazyCompact, SmartString};

use super::ports::NodeInput;
use crate::{
	api::{InitNodeError, PipelineData},
	labels::{PipelineName, PipelineNodeID, PipelinePortID},
};

/// A node specification in a [`PipelinePrepareError`]
#[derive(Debug)]
pub enum PipelineErrorNode {
	/// The pipeline's output node
	PipelineOutput,

	/// The pipeline's input node
	PipelineInput,

	/// An inline node
	Inline,

	/// A named node created by the user
	Named(PipelineNodeID),
}

/// An error we encounter when a pipeline spec is invalid
#[derive(Debug)]
pub enum PipelinePrepareError<DataType: PipelineData> {
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
		/// The node id that doesn't exist
		node: PipelineNodeID,
		/// We tried to connect to `node` from this input.
		caused_by: NodeInput,
	},

	/// There is no node named `node` in this pipeline.
	NoNodeAfter {
		/// The node id that doesn't exist
		node: PipelineNodeID,

		/// We tried to specify `node` in this node's `after` parameter
		caused_by_after_in: PipelineNodeID,
	},

	/// `node` has no input named `input`.
	/// This is triggered when we specify an input that doesn't exist.
	NoNodeInput {
		/// The node we tried to reference
		node: PipelineErrorNode,
		/// The input name that doesn't exist
		input: PipelinePortID,
	},

	/// `node` has no output named `output`.
	NoNodeOutput {
		/// The node we tried to connect to
		node: PipelineErrorNode,
		/// The output name that doesn't exist
		output: PipelinePortID,
		// The node input we tried to connect to `output`
		//caused_by: NodeInput,
	},

	/// We tried to connect `input` to `output`,
	/// but their types don't match.
	TypeMismatch {
		/// The output we tried to connect
		output: (PipelineErrorNode, PipelinePortID),

		/// the type of this output
		output_type: <DataType as PipelineData>::DataStubType,

		/// The input we tried to connect
		input: NodeInput,
	},

	/// This graph has a cycle containing `node`
	HasCycle,

	/// A `Pipeline` node in this graph references an unknown pipeline
	NoSuchPipeline {
		/// The Pipeline node with a bad pipeline
		node: PipelineNodeID,

		/// The bad pipeline
		pipeline: PipelineName,
	},

	/// We tried to create a node with an unrecognized type
	InvalidNodeType {
		/// The node that was invalid
		node: PipelineNodeID,

		///The invalid type
		bad_type: SmartString<LazyCompact>,
	},

	/// We encountered an [`InitNodeError`] while building a pipeline
	InitNodeError {
		/// The error we encountered
		error: InitNodeError,
	},
}

impl<DataType: PipelineData> Display for PipelinePrepareError<DataType> {
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
			Self::NoNodeAfter {
				node,
				caused_by_after_in,
			} => {
				writeln!(
					f,
					"PipelinePrepareError: No such node `{node:?}`. Caused by `after` in node `{caused_by_after_in:?}`."
				)
			}
			Self::NoNodeInput { node, input } => {
				writeln!(
					f,
					"PipelinePrepareError: Node `{node:?}` has no input `{input}`"
				)
			}
			Self::NoNodeOutput { node, output } => {
				writeln!(
					f,
					"PipelinePrepareError: Node `{node:?}` has no output `{output}`."
				)
			}
			Self::TypeMismatch {
				output,
				input,
				output_type,
			} => {
				writeln!(
					f,
					"PipelinePrepareError: `{output:?}` produces datatype {output_type:?}, but `{input:?}` cannot consume it."
				)
			}
			Self::HasCycle => {
				writeln!(f, "PipelinePrepareError: This pipeline has a cycle.")
			}
			Self::NoSuchPipeline { node, pipeline } => {
				writeln!(
					f,
					"PipelinePrepareError: Node {node} references an unknown pipeline {pipeline}"
				)
			}
			Self::InvalidNodeType { node, bad_type } => {
				writeln!(f, "Node {node} has invalid type {bad_type}")
			}
			Self::InitNodeError { .. } => {
				writeln!(f, "Encountered a InitNodeError")
			}
		}
	}
}

impl<DataType: PipelineData> Error for PipelinePrepareError<DataType> {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::CouldNotOpenFile { error } => Some(error),
			Self::CouldNotReadFile { error } => Some(error),
			Self::CouldNotParseFile { error } => Some(error),
			Self::InitNodeError { error } => Some(error),
			_ => None,
		}
	}
}

impl<DataType: PipelineData> From<InitNodeError> for PipelinePrepareError<DataType> {
	fn from(error: InitNodeError) -> Self {
		Self::InitNodeError { error }
	}
}
