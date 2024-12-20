use std::collections::BTreeMap;

use copper_piper::base::{NodeDispatcher, RegisterNodeError};

mod additem;
mod constant;
mod hash;
mod ifnone;

/// Register all nodes in this module into the given runner.
pub fn register(dispatcher: &mut NodeDispatcher) -> Result<(), RegisterNodeError> {
	dispatcher.register_node("IfNone", BTreeMap::new(), Box::new(ifnone::IfNone {}))?;
	dispatcher.register_node("Hash", BTreeMap::new(), Box::new(hash::Hash {}))?;
	dispatcher.register_node("Constant", BTreeMap::new(), Box::new(constant::Constant {}))?;
	dispatcher.register_node("AddItem", BTreeMap::new(), Box::new(additem::AddItem {}))?;

	return Ok(());
}
