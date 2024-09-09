//! Pipeline nodes for processing audio files

use pipelined_node_base::{
	base::{NodeDispatcher, RegisterNodeError},
	data::CopperData,
	CopperContext,
};
use std::collections::BTreeMap;

mod extractcovers;
mod extracttags;
mod striptags;

/// Register all nodes in this module into the given dispatcher
pub fn register(
	dispatcher: &mut NodeDispatcher<CopperData, CopperContext>,
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
