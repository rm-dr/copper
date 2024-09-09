//! Pipeline nodes for processing audio files

use copper_pipelined::{
	base::{NodeDispatcher, RegisterNodeError},
	data::PipeData,
	CopperContext,
};
use std::collections::BTreeMap;

mod extractcovers;
mod extracttags;
mod striptags;

/// Register all nodes in this module into the given dispatcher
pub fn register(
	dispatcher: &mut NodeDispatcher<PipeData, CopperContext>,
) -> Result<(), RegisterNodeError> {
	dispatcher
		.register_node("StripTags", BTreeMap::new(), &|ctx, params, _| {
			Ok(Box::new(striptags::StripTags::new(ctx, params)?))
		})
		.unwrap();

	dispatcher
		.register_node("ExtractCovers", BTreeMap::new(), &|ctx, params, _| {
			Ok(Box::new(extractcovers::ExtractCovers::new(ctx, params)?))
		})
		.unwrap();

	dispatcher
		.register_node("ExtractTags", BTreeMap::new(), &|ctx, params, _| {
			Ok(Box::new(extracttags::ExtractTags::new(ctx, params)?))
		})
		.unwrap();

	return Ok(());
}
