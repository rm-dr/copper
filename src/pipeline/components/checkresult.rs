use super::{PipelineInput, PipelineNodeLabel, PipelineOutput, PipelinePortLabel};
use crate::pipeline::data::PipelineDataType;

/// The result of a [`Pipeline::check()`].
#[derive(Debug)]
pub enum PipelineCheckResult {
	/// This pipeline is good to go.
	Ok,

	/// There is no node named `node` in this pipeline
	/// We tried to connect this node from `caused_by_input`.
	NoNode {
		node: PipelineNodeLabel,
		caused_by_input: PipelineInput,
	},

	/// `node` has no input named `input_name`.
	/// This is triggered when we specify an input that doesn't exist.
	NoNodeInput {
		node: PipelineNodeLabel,
		input_name: PipelinePortLabel,
	},

	/// `node` has no output named `output_name`.
	/// We tried to connect this output from `caused_by_input`.
	NoNodeOutput {
		node: PipelineNodeLabel,
		output_name: PipelinePortLabel,
		caused_by_input: PipelineInput,
	},

	/// This pipeline has no input named `input_name`.
	/// We tried to connect to this input from `caused_by_input`.
	NoPipelineInput {
		pipeline_input_name: PipelinePortLabel,
		caused_by_input: PipelineInput,
	},

	/// This pipeline has no output named `output_name`.
	NoPipelineOutput {
		pipeline_output_name: PipelinePortLabel,
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
	HasCycle { node: PipelineNodeLabel },
}
