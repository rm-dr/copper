pub mod base;
pub mod data;
pub mod helpers;

use base::PipelineJobContext;
use copper_storaged::client::BlockingStoragedClient;
use data::PipeData;
use smartstring::{LazyCompact, SmartString};
use std::{collections::BTreeMap, sync::Arc};

pub struct CopperContext {
	/// The maximum size, in bytes, of a blob channel fragment
	pub blob_fragment_size: u64,
	pub input: BTreeMap<SmartString<LazyCompact>, PipeData>,

	pub storaged_client: Arc<dyn BlockingStoragedClient>,
}

impl PipelineJobContext<PipeData> for CopperContext {
	fn get_input(&self) -> &BTreeMap<SmartString<LazyCompact>, PipeData> {
		&self.input
	}
}
