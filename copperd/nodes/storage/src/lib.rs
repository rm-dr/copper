//! Pipeline nodes for processing audio files

use copper_pipelined::{
	base::{NodeDispatcher, RegisterNodeError},
	data::PipeData,
	CopperContext, JobRunResult,
};
use copper_storage::database::base::client::StorageDatabaseClient;
use std::collections::BTreeMap;

mod additem;

/// Register all nodes in this module into the given dispatcher
pub fn register<StorageClient: StorageDatabaseClient>(
	dispatcher: &mut NodeDispatcher<JobRunResult, PipeData, CopperContext<StorageClient>>,
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
