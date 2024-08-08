//! Pipeline node implementations

//#![warn(missing_docs)]

pub mod data;
pub mod helpers;

use std::sync::Arc;
use ufo_ds_impl::local::LocalDataset;
use ufo_pipeline::api::PipelineJobContext;

#[derive(Clone)]
pub struct UFOContext {
	// Hard-code LocalDataset for now,
	// TODO: this should be some form of "generic dataset" later.
	//
	// Maybe don't provide a dataset, but a way to *get* datasets?
	// (maindb ref?)
	pub dataset: Arc<LocalDataset>,

	/// The maximum size, in bytes, of a blob channel fragment
	pub blob_fragment_size: u64,
}

impl PipelineJobContext for UFOContext {}
