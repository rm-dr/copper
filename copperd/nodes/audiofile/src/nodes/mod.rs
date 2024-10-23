//! Pipeline nodes for processing audio files

use copper_pipelined::{
	base::{NodeDispatcher, RegisterNodeError},
	data::PipeData,
	CopperContext, JobRunResult,
};
use copper_storage::database::base::client::StorageDatabaseClient;
use std::collections::BTreeMap;

mod extractcovers;
mod extracttags;
mod striptags;

/// Register all nodes in this module into the given dispatcher
pub fn register<StorageClient: StorageDatabaseClient>(
	dispatcher: &mut NodeDispatcher<JobRunResult, PipeData, CopperContext<StorageClient>>,
) -> Result<(), RegisterNodeError> {
	dispatcher
		.register_node("StripTags", BTreeMap::new(), &|| {
			Box::new(striptags::StripTags {})
		})
		.unwrap();

	dispatcher
		.register_node("ExtractCovers", BTreeMap::new(), &|| {
			Box::new(extractcovers::ExtractCovers {})
		})
		.unwrap();

	dispatcher
		.register_node("ExtractTags", BTreeMap::new(), &|| {
			Box::new(extracttags::ExtractTags {})
		})
		.unwrap();

	return Ok(());
}
