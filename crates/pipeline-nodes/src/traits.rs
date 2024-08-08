//! Helper traits

use ufo_db_metastore::data::MetastoreDataStub;
use ufo_pipeline::labels::PipelinePortLabel;

use crate::{nodetype::UFONodeType, UFOContext};

/// Information about a node's inputs & outputs
pub trait UFONode {
	fn n_inputs(stub: &UFONodeType, ctx: &UFOContext) -> usize;

	fn input_compatible_with(
		stub: &UFONodeType,
		ctx: &UFOContext,
		input_idx: usize,
		input_type: MetastoreDataStub,
	) -> bool;

	fn input_default_type(
		stub: &UFONodeType,
		ctx: &UFOContext,
		input_idx: usize,
	) -> MetastoreDataStub;

	fn input_with_name(
		stub: &UFONodeType,
		ctx: &UFOContext,
		input_name: &PipelinePortLabel,
	) -> Option<usize>;

	fn n_outputs(stub: &UFONodeType, ctx: &UFOContext) -> usize;

	fn output_type(stub: &UFONodeType, ctx: &UFOContext, output_idx: usize) -> MetastoreDataStub;

	fn output_with_name(
		stub: &UFONodeType,
		ctx: &UFOContext,
		output_name: &PipelinePortLabel,
	) -> Option<usize>;
}

/// A shortcut implementation for nodes that provide a static set of inputs & outputs
pub trait UFOStaticNode {
	fn inputs() -> &'static [(&'static str, MetastoreDataStub)];
	fn outputs() -> &'static [(&'static str, MetastoreDataStub)];
}

impl<T> UFONode for T
where
	T: UFOStaticNode,
{
	fn n_inputs(_stub: &UFONodeType, _ctx: &UFOContext) -> usize {
		Self::inputs().len()
	}

	fn input_compatible_with(
		stub: &UFONodeType,
		ctx: &UFOContext,
		input_idx: usize,
		input_type: MetastoreDataStub,
	) -> bool {
		Self::input_default_type(stub, ctx, input_idx) == input_type
	}

	fn input_with_name(
		_stub: &UFONodeType,
		_ctx: &UFOContext,
		input_name: &PipelinePortLabel,
	) -> Option<usize> {
		Self::inputs()
			.iter()
			.enumerate()
			.find(|(_, (n, _))| PipelinePortLabel::from(*n) == *input_name)
			.map(|(x, _)| x)
	}

	fn input_default_type(
		_stub: &UFONodeType,
		_ctx: &UFOContext,
		input_idx: usize,
	) -> MetastoreDataStub {
		Self::inputs().get(input_idx).unwrap().1
	}

	fn n_outputs(_stub: &UFONodeType, _ctx: &UFOContext) -> usize {
		Self::outputs().len()
	}

	fn output_type(_stub: &UFONodeType, _ctx: &UFOContext, output_idx: usize) -> MetastoreDataStub {
		Self::outputs().get(output_idx).unwrap().1
	}

	fn output_with_name(
		_stub: &UFONodeType,
		_ctx: &UFOContext,
		output_name: &PipelinePortLabel,
	) -> Option<usize> {
		Self::outputs()
			.iter()
			.enumerate()
			.find(|(_, (n, _))| PipelinePortLabel::from(*n) == *output_name)
			.map(|(x, _)| x)
	}
}
