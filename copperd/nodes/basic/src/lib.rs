use std::collections::BTreeMap;

use copper_itemdb::client::base::client::ItemdbClient;
use copper_pipelined::{
	base::{NodeDispatcher, RegisterNodeError},
	data::PipeData,
	CopperContext, JobRunResult,
};

mod additem;
mod constant;
mod hash;
mod ifnone;

/// Register all nodes in this module into the given runner.
pub fn register<Itemdb: ItemdbClient>(
	dispatcher: &mut NodeDispatcher<JobRunResult, PipeData, CopperContext<Itemdb>>,
) -> Result<(), RegisterNodeError> {
	dispatcher.register_node("IfNone", BTreeMap::new(), &|| Box::new(ifnone::IfNone {}))?;
	dispatcher.register_node("Hash", BTreeMap::new(), &|| Box::new(hash::Hash {}))?;
	dispatcher.register_node("Constant", BTreeMap::new(), &|| {
		Box::new(constant::Constant {})
	})?;

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
