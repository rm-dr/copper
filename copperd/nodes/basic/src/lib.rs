use std::collections::BTreeMap;

use copper_pipelined::{
	base::{NodeDispatcher, RegisterNodeError},
	data::PipeData,
	CopperContext, JobRunResult,
};
use copper_storage::database::base::client::StorageDatabaseClient;

mod constant;
mod hash;
mod ifnone;

/// Register all nodes in this module into the given runner.
pub fn register<StorageClient: StorageDatabaseClient>(
	dispatcher: &mut NodeDispatcher<JobRunResult, PipeData, CopperContext<StorageClient>>,
) -> Result<(), RegisterNodeError> {
	dispatcher.register_node("IfNone", BTreeMap::new(), &|| Box::new(ifnone::IfNone {}))?;
	dispatcher.register_node("Hash", BTreeMap::new(), &|| Box::new(hash::Hash {}))?;
	dispatcher.register_node("Constant", BTreeMap::new(), &|| {
		Box::new(constant::Constant {})
	})?;

	return Ok(());
}
