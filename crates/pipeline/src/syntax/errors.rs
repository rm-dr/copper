use smartstring::{LazyCompact, SmartString};

use super::{
	labels::{PipelineNode, PipelinePortLabel},
	ports::{NodeInput, NodeOutput},
};
use crate::data::PipelineDataType;

#[derive(Debug)]
pub enum PipelinePrepareError {
	/// We tried to create a node with a reserved name
	NodeHasReservedName { node: SmartString<LazyCompact> },

	/// There is no node named `node` in this pipeline
	/// We tried to connect this node from `caused_by`.
	NoNode {
		node: PipelineNode,
		caused_by: NodeInput,
	},

	/// `node` has no input named `input`.
	/// This is triggered when we specify an input that doesn't exist.
	NoNodeInput {
		node: PipelineNode,
		input: PipelinePortLabel,
	},

	/// `node` has no output named `output`.
	/// We tried to connect this output from `caused_by`.
	NoNodeOutput {
		node: PipelineNode,
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
