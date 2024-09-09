use std::collections::BTreeMap;

use pipelined_node_base::{
	base::{NodeDispatcher, RegisterNodeError},
	data::CopperData,
	CopperContext,
};

// TODO: move to another lib

mod constant;
mod hash;
mod ifnone;

/// Register all nodes in this module into the given runner.
pub fn register(
	dispatcher: &mut NodeDispatcher<CopperData, CopperContext>,
) -> Result<(), RegisterNodeError> {
	dispatcher.register_node("Constant", BTreeMap::new(), &|_ctx, params, _| {
		Ok(Box::new(constant::Constant::new(params)?))
	})?;

	dispatcher.register_node("Hash", BTreeMap::new(), &|_ctx, params, _| {
		Ok(Box::new(hash::Hash::new(params)?))
	})?;

	dispatcher.register_node("IfNone", BTreeMap::new(), &|_ctx, params, _| {
		Ok(Box::new(ifnone::IfNone::new(params)?))
	})?;

	return Ok(());
}
