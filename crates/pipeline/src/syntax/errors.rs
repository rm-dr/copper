use std::{error::Error, fmt::Display};
use ufo_util::data::PipelineDataType;

use super::{
	labels::{PipelineNodeLabel, PipelinePortLabel},
	ports::{NodeInput, NodeOutput},
};

#[derive(Debug)]
pub enum PipelineErrorNode {
	Pipeline,
	Named(PipelineNodeLabel),
}

#[derive(Debug)]
pub enum PipelinePrepareError {
	/// We could not open a pipeline spec file
	CouldNotOpenFile { error: std::io::Error },

	/// We could not read a pipeline spec file
	CouldNotReadFile { error: std::io::Error },

	/// We could not parse a pipeline spec file
	CouldNotParseFile { error: toml::de::Error },

	/// There is no node named `node` in this pipeline
	/// We tried to connect this node from `caused_by`.
	NoNode {
		node: PipelineNodeLabel,
		caused_by: NodeInput,
	},

	/// `node` has no input named `input`.
	/// This is triggered when we specify an input that doesn't exist.
	NoNodeInput {
		node: PipelineErrorNode,
		input: PipelinePortLabel,
	},

	/// `node` has no output named `output`.
	/// We tried to connect this output from `caused_by`.
	NoNodeOutput {
		node: PipelineErrorNode,
		output: PipelinePortLabel,
		caused_by: NodeInput,
	},

	/// We tried to connect `input` to `output`,
	/// but their types don't match.
	TypeMismatch {
		output: NodeOutput,
		input: NodeInput,
	},

	/// We tried to connect an inline type to `input`,
	/// but their types don't match.
	InlineTypeMismatch {
		inline_type: PipelineDataType,
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
