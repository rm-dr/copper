//! Pipeline nodes for processing audio files

use copper_pipelined::{
	base::{NodeDispatcher, RegisterNodeError},
	data::PipeData,
	CopperContext, JobRunResult,
};
use std::collections::BTreeMap;

mod additem;

/// Register all nodes in this module into the given dispatcher
pub fn register(
	dispatcher: &mut NodeDispatcher<JobRunResult, PipeData, CopperContext>,
) -> Result<(), RegisterNodeError> {
	dispatcher.register_node("AddItem", BTreeMap::new(), &|| {
		Box::new(additem::AddItem {})
	})?;

	/*
	dispatcher.register_node(
		"FindItem",
		BTreeMap::new(),
		&|ctx, params, _| Ok(Box::new(finditem::FindItem::new(ctx, params)?)),
	)?;
	*/

	return Ok(());
}
