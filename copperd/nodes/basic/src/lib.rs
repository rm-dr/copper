use std::collections::BTreeMap;

use copper_pipelined::{
	base::{NodeDispatcher, RegisterNodeError},
	data::PipeData,
	CopperContext,
};

mod constant;
mod hash;
mod ifnone;

/// Register all nodes in this module into the given runner.
pub fn register(
	dispatcher: &mut NodeDispatcher<PipeData, CopperContext>,
) -> Result<(), RegisterNodeError> {
	dispatcher.register_node("IfNone", BTreeMap::new(), &|| Box::new(ifnone::IfNone {}))?;
	dispatcher.register_node("Hash", BTreeMap::new(), &|| Box::new(hash::Hash {}))?;
	dispatcher.register_node("Constant", BTreeMap::new(), &|| {
		Box::new(constant::Constant {})
	})?;

	return Ok(());
}
