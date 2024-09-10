use std::collections::BTreeMap;

use copper_pipelined::{
	base::{NodeDispatcher, RegisterNodeError},
	data::PipeData,
	CopperContext,
};

// TODO: move to another lib

mod constant;
mod hash;
mod ifnone;

/// Register all nodes in this module into the given runner.
pub fn register(
	dispatcher: &mut NodeDispatcher<PipeData, CopperContext>,
) -> Result<(), RegisterNodeError> {
	dispatcher.register_node("Constant", BTreeMap::new(), &|_ctx, params, _| {
		Ok(Box::new(constant::Constant::new(params)?))
	})?;

	dispatcher.register_node("Hash", BTreeMap::new(), &|ctx, params, _| {
		Ok(Box::new(hash::Hash::new(ctx, params)?))
	})?;

	dispatcher.register_node("IfNone", BTreeMap::new(), &|_ctx, params, _| {
		Ok(Box::new(ifnone::IfNone::new(params)?))
	})?;

	return Ok(());
}
