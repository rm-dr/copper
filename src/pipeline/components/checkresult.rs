use smartstring::{LazyCompact, SmartString};

use super::{PipelineInput, PipelineOutput};
use crate::pipeline::data::PipelineDataType;

/// The result of a [`Pipeline::check()`].
#[derive(Debug)]
pub enum PipelineCheckResult {
	/// This pipeline is good to go.
	Ok {
		/// A vector of all nodes in this pipeline in topological order:
		/// each node is ordered before its successors.
		topo: Vec<SmartString<LazyCompact>>,
	},

	/// There is no node named `node` in this pipeline
	/// We tried to connect this node from `caused_by_input`.
	NoNode {
		node: SmartString<LazyCompact>,
		caused_by_input: PipelineInput,
	},

	/// `node` has no input named `input_name`.
	/// This is triggered when we specify an input that doesn't exist.
	NoNodeInput {
		node: SmartString<LazyCompact>,
		input_name: SmartString<LazyCompact>,
	},

	/// `node` has no output named `output_name`.
	/// We tried to connect this output from `caused_by_input`.
	NoNodeOutput {
		node: SmartString<LazyCompact>,
		output_name: SmartString<LazyCompact>,
		caused_by_input: PipelineInput,
	},

	/// This pipeline has no input named `input_name`.
	/// We tried to connect to this input from `caused_by_input`.
	NoPipelineInput {
		pipeline_input_name: SmartString<LazyCompact>,
		caused_by_input: PipelineInput,
	},

	/// This pipeline has no output named `output_name`.
	NoPipelineOutput {
		pipeline_output_name: SmartString<LazyCompact>,
	},

	/// We tried to connect `input` to `output`,
	/// but their types don't match.
	TypeMismatch {
		output: PipelineOutput,
		input: PipelineInput,
	},

	/// We tried to connect an inline type to `input`,
	/// but their types don't match.
	InlineTypeMismatch {
		inline_type: PipelineDataType,
		input: PipelineInput,
	},

	/// This graph has a cycle containing `node`
	HasCycle { node: SmartString<LazyCompact> },
}
