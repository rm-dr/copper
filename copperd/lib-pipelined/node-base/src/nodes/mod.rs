use pipelined_pipeline::dispatcher::{NodeDispatcher, RegisterNodeError};
use std::collections::BTreeMap;

use crate::{data::CopperData, CopperContext};

mod constant;
mod hash;
mod ifnone;

// TODO: move these to ds-impl once we fix the "generic dataset" problem
// (cannot do this now, it will cause a crate dependency cycle)
// mod additem;
// mod finditem;

/// Register all nodes in this module into the given runner.
pub fn register(
    dispatcher: &mut NodeDispatcher<CopperData, CopperContext>,
) -> Result<(), RegisterNodeError> {
    dispatcher.register_node(
        "Constant",
        BTreeMap::new(),
        &|_ctx, params, _| Ok(Box::new(constant::Constant::new(params)?)),
        &|_ctx, params, _| Ok(Box::new(constant::Constant::new(params)?)),
    )?;

    dispatcher.register_node(
        "Hash",
        BTreeMap::new(),
        &|_ctx, params, _| Ok(Box::new(hash::Hash::new(params)?)),
        &|_ctx, params, _| Ok(Box::new(hash::Hash::new(params)?)),
    )?;

    dispatcher.register_node(
        "IfNone",
        BTreeMap::new(),
        &|_ctx, params, _| Ok(Box::new(ifnone::IfNone::new(params)?)),
        &|_ctx, params, _| Ok(Box::new(ifnone::IfNone::new(params)?)),
    )?;

    /*
    dispatcher.register_node(
        "AddItem",
        BTreeMap::new(),
        &|ctx, params, _| Ok(Box::new(additem::AddItemInfo::new(ctx, params)?)),
        &|ctx, params, _| Ok(Box::new(additem::AddItem::new(ctx, params)?)),
    )?;

    dispatcher.register_node(
        "FindItem",
        BTreeMap::new(),
        &|ctx, params, _| Ok(Box::new(finditem::FindItemInfo::new(ctx, params)?)),
        &|ctx, params, _| Ok(Box::new(finditem::FindItem::new(ctx, params)?)),
    )?;
    */

    return Ok(());
}
