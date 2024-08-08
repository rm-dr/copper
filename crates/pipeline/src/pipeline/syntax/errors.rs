//! Errors we can encounter when parsing a pipeline

use std::{error::Error, fmt::Display};

use super::ports::NodeInput;
use crate::{
	api::PipelineDataStub,
	labels::{PipelineLabel, PipelineNodeLabel, PipelinePortLabel},
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
	Named(PipelineNodeLabel),
}

/// An error we encounter when a pipeline spec is invalid
#[derive(Debug)]
pub enum PipelinePrepareError<DataStubType: PipelineDataStub> {
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

	/// There is no node named `node` in this pipeline.
	NoNodeAfter {
		/// The node label that doesn't exist
		node: PipelineNodeLabel,

		/// We tried to specify `node` in this node's `after` parameter
		caused_by_after_in: PipelineNodeLabel,
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
		// The node input we tried to connect to `output`
		//caused_by: NodeInput,
	},

	/// We tried to connect `input` to `output`,
	/// but their types don't match.
	TypeMismatch {
		/// The output we tried to connect
		output: (PipelineErrorNode, PipelinePortLabel),

		/// the type of this output
		output_type: DataStubType,

		/// The input we tried to connect
		input: NodeInput,
	},

	/// We tried to use a node with multiple outputs inline
	BadInlineNode {
		/// The input we connected to
		input: NodeInput,
	},

	/// This graph has a cycle containing `node`
	HasCycle,

	/// A `Pipeline` node in this graph references an unknown pipeline
	NoSuchPipeline {
		/// The Pipeline node with a bad pipeline
		node: PipelineNodeLabel,
		/// The bad pipeline
		pipeline: PipelineLabel,
	},
}

impl<DataStubType: PipelineDataStub> Display for PipelinePrepareError<DataStubType> {
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
			Self::BadInlineNode { input } => {
				writeln!(f, "PipelinePrepareError: Inline node in `{input:?}` doesn't have exactly one argument.")
			}
			Self::NoNodeOutput {
				node,
				output,
				//caused_by,
			} => {
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
		}
	}
}

impl<DataStubType: PipelineDataStub> Error for PipelinePrepareError<DataStubType> {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::CouldNotOpenFile { error } => Some(error),
			Self::CouldNotReadFile { error } => Some(error),
			Self::CouldNotParseFile { error } => Some(error),
			_ => None,
		}
	}
}
