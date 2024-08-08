//! Pipeline node implementations

//#![warn(missing_docs)]

pub mod data;
pub mod helpers;
pub mod nodes;

use copper_ds_impl::local::LocalDataset;
use data::CopperData;
use smartstring::{LazyCompact, SmartString};
use std::{collections::BTreeMap, sync::Arc};
use copper_pipeline::api::PipelineJobContext;

#[derive(Clone)]
pub struct CopperContext {
	// Hard-code LocalDataset for now,
	// TODO: this should be some form of "generic dataset" later.
	//
	// Maybe don't provide a dataset, but a way to *get* datasets?
	// (maindb ref?)
	pub dataset: Arc<LocalDataset>,

	/// The maximum size, in bytes, of a blob channel fragment
	pub blob_fragment_size: u64,

	pub input: BTreeMap<SmartString<LazyCompact>, CopperData>,
}

impl PipelineJobContext<CopperData> for CopperContext {
	fn get_input(&self) -> &BTreeMap<SmartString<LazyCompact>, CopperData> {
		&self.input
	}
}
