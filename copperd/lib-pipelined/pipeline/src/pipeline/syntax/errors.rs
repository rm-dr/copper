//! Errors we can encounter when parsing a pipeline

use std::{error::Error, fmt::Display};

use smartstring::{LazyCompact, SmartString};

use crate::{
	base::{InitNodeError, PipelineData},
	labels::{PipelineNodeID, PipelinePortID},
};

/// An error we encounter when a pipeline spec is invalid
#[derive(Debug)]
pub enum PipelineBuildError<DataType: PipelineData> {
	/// An edge references a node, but it doesn't exist
	NoNode {
		/// The edge that references an invalid node
		edge_id: SmartString<LazyCompact>,

		/// The node id that doesn't exist
		invalid_node_id: PipelineNodeID,
	},

	/// We tried to connect `input` to `output`,
	/// but their types don't match.
	TypeMismatch {
		/// The offending edge
		edge_id: SmartString<LazyCompact>,

		/// The source type
		source_type: <DataType as PipelineData>::DataStubType,

		/// The incompatible target type
		target_type: <DataType as PipelineData>::DataStubType,
	},

	/// This pipeline has a cycle and is thus invalid
	HasCycle,

	/// `node` has no output port named `input`.
	/// This is triggered when we specify an input that doesn't exist.
	NoSuchOutputPort {
		/// The responsible edge
		edge_id: SmartString<LazyCompact>,
		/// The node we tried to reference
		node: PipelineNodeID,
		/// The port that doesn't exist
		invalid_port: PipelinePortID,
	},

	/// `node` has no input port named `port`.
	/// This is triggered when we specify an input that doesn't exist.
	NoSuchInputPort {
		/// The responsible edge
		edge_id: SmartString<LazyCompact>,
		/// The node we tried to reference
		node: PipelineNodeID,
		/// The port name that doesn't exist
		invalid_port: PipelinePortID,
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

impl<DataType: PipelineData> Display for PipelineBuildError<DataType> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::NoNode {
				edge_id,
				invalid_node_id,
			} => {
				writeln!(
					f,
					"edge `{edge_id}` references a node `{invalid_node_id}` that doesn't exist"
				)
			}

			Self::TypeMismatch {
				edge_id,
				source_type,
				target_type,
			} => {
				writeln!(
					f,
					"edge `{edge_id}` connects incompatible types `{source_type:?}` and `{target_type:?}`"
				)
			}

			Self::HasCycle => {
				writeln!(f, "this pipeline has a cycle")
			}

			Self::NoSuchInputPort {
				edge_id,
				node,
				invalid_port,
			} => {
				writeln!(
					f,
					"edge `{edge_id}` references invalid input port `{invalid_port}` on node `{node}`"
				)
			}

			Self::NoSuchOutputPort {
				edge_id,
				node,
				invalid_port,
			} => {
				writeln!(
					f,
					"edge `{edge_id}` references invalid output port `{invalid_port}` on node `{node}`"
				)
			}

			Self::InvalidNodeType { node, bad_type } => {
				writeln!(f, "node `{node}` has invalid type `{bad_type}`")
			}

			Self::InitNodeError { .. } => {
				writeln!(f, "could not initialize node")
			}
		}
	}
}

impl<DataType: PipelineData> Error for PipelineBuildError<DataType> {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::InitNodeError { error } => Some(error),
			_ => None,
		}
	}
}

impl<DataType: PipelineData> From<InitNodeError> for PipelineBuildError<DataType> {
	fn from(error: InitNodeError) -> Self {
		Self::InitNodeError { error }
	}
}
