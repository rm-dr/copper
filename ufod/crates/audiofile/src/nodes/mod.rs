//! Pipeline nodes for processing audio files

use std::collections::BTreeMap;
use ufo_node_base::{data::UFOData, UFOContext};
use ufo_pipeline::dispatcher::{NodeDispatcher, RegisterNodeError};

mod extractcovers;
mod extracttags;
mod striptags;

/// Register all nodes in this module into the given dispatcher
pub fn register(
	dispatcher: &mut NodeDispatcher<UFOData, UFOContext>,
) -> Result<(), RegisterNodeError> {
	dispatcher
		.register_node(
			"StripTags",
			BTreeMap::new(),
			&|_ctx, params, _| Ok(Box::new(striptags::StripTagsInfo::new(params)?)),
			&|ctx, params, _| Ok(Box::new(striptags::StripTags::new(ctx, params)?)),
		)
		.unwrap();

	dispatcher
		.register_node(
			"ExtractCovers",
			BTreeMap::new(),
			&|_ctx, params, _| Ok(Box::new(extractcovers::ExtractCoversInfo::new(params)?)),
			&|ctx, params, _| Ok(Box::new(extractcovers::ExtractCovers::new(ctx, params)?)),
		)
		.unwrap();

	dispatcher
		.register_node(
			"ExtractTags",
			BTreeMap::new(),
			&|_ctx, params, _| Ok(Box::new(extracttags::ExtractTagsInfo::new(params)?)),
			&|ctx, params, _| Ok(Box::new(extracttags::ExtractTags::new(ctx, params)?)),
		)
		.unwrap();

	return Ok(());
}
