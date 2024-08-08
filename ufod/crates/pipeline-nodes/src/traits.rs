//! Helper traits

use ufo_pipeline::labels::PipelinePortID;

use crate::{
	data::UFODataStub,
	nodetype::{UFONodeType, UFONodeTypeError},
	UFOContext,
};

/// Information about a node's inputs & outputs
pub trait UFONode {
	fn n_inputs(stub: &UFONodeType, ctx: &UFOContext) -> Result<usize, UFONodeTypeError>;

	fn input_compatible_with(
		stub: &UFONodeType,
		ctx: &UFOContext,
		input_idx: usize,
		input_type: UFODataStub,
	) -> Result<bool, UFONodeTypeError>;

	fn input_default_type(
		stub: &UFONodeType,
		ctx: &UFOContext,
		input_idx: usize,
	) -> Result<UFODataStub, UFONodeTypeError>;

	fn input_with_name(
		stub: &UFONodeType,
		ctx: &UFOContext,
		input_name: &PipelinePortID,
	) -> Result<Option<usize>, UFONodeTypeError>;

	fn n_outputs(stub: &UFONodeType, ctx: &UFOContext) -> Result<usize, UFONodeTypeError>;

	fn output_type(
		stub: &UFONodeType,
		ctx: &UFOContext,
		output_idx: usize,
	) -> Result<UFODataStub, UFONodeTypeError>;

	fn output_with_name(
		stub: &UFONodeType,
		ctx: &UFOContext,
		output_name: &PipelinePortID,
	) -> Result<Option<usize>, UFONodeTypeError>;
}

/// A shortcut implementation for nodes that provide a static set of inputs & outputs
pub trait UFOStaticNode {
	fn inputs() -> &'static [(&'static str, UFODataStub)];
	fn outputs() -> &'static [(&'static str, UFODataStub)];
}

impl<T> UFONode for T
where
	T: UFOStaticNode,
{
	fn n_inputs(_stub: &UFONodeType, _ctx: &UFOContext) -> Result<usize, UFONodeTypeError> {
		Ok(Self::inputs().len())
	}

	fn input_compatible_with(
		stub: &UFONodeType,
		ctx: &UFOContext,
		input_idx: usize,
		input_type: UFODataStub,
	) -> Result<bool, UFONodeTypeError> {
		Ok(Self::input_default_type(stub, ctx, input_idx)? == input_type)
	}

	fn input_with_name(
		_stub: &UFONodeType,
		_ctx: &UFOContext,
		input_name: &PipelinePortID,
	) -> Result<Option<usize>, UFONodeTypeError> {
		Ok(Self::inputs()
			.iter()
			.enumerate()
			.find(|(_, (n, _))| PipelinePortID::new(n) == *input_name)
			.map(|(x, _)| x))
	}

	fn input_default_type(
		_stub: &UFONodeType,
		_ctx: &UFOContext,
		input_idx: usize,
	) -> Result<UFODataStub, UFONodeTypeError> {
		Ok(Self::inputs().get(input_idx).unwrap().1)
	}

	fn n_outputs(_stub: &UFONodeType, _ctx: &UFOContext) -> Result<usize, UFONodeTypeError> {
		Ok(Self::outputs().len())
	}

	fn output_type(
		_stub: &UFONodeType,
		_ctx: &UFOContext,
		output_idx: usize,
	) -> Result<UFODataStub, UFONodeTypeError> {
		Ok(Self::outputs().get(output_idx).unwrap().1)
	}

	fn output_with_name(
		_stub: &UFONodeType,
		_ctx: &UFOContext,
		output_name: &PipelinePortID,
	) -> Result<Option<usize>, UFONodeTypeError> {
		Ok(Self::outputs()
			.iter()
			.enumerate()
			.find(|(_, (n, _))| PipelinePortID::new(n) == *output_name)
			.map(|(x, _)| x))
	}
}
