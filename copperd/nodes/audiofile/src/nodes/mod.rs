//! Pipeline nodes for processing audio files
use copper_piper::base::{NodeDispatcher, RegisterNodeError};
use std::collections::BTreeMap;

mod extractcovers;
mod extracttags;
mod striptags;

/// Register all nodes in this module into the given dispatcher
pub fn register(dispatcher: &mut NodeDispatcher) -> Result<(), RegisterNodeError> {
	dispatcher
		.register_node(
			"StripTags",
			BTreeMap::new(),
			Box::new(striptags::StripTags {}),
		)
		.unwrap();

	dispatcher
		.register_node(
			"ExtractCovers",
			BTreeMap::new(),
			Box::new(extractcovers::ExtractCovers {}),
		)
		.unwrap();

	dispatcher
		.register_node(
			"ExtractTags",
			BTreeMap::new(),
			Box::new(extracttags::ExtractTags {}),
		)
		.unwrap();

	return Ok(());
}
