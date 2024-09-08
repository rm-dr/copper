//! Pipeline node implementations

//#![warn(missing_docs)]

pub mod base;
pub mod data;
pub mod helpers;
pub mod nodes;

use base::PipelineJobContext;
use data::CopperData;
use smartstring::{LazyCompact, SmartString};
use std::collections::BTreeMap;

#[derive(Clone)]
pub struct CopperContext {
	/// The maximum size, in bytes, of a blob channel fragment
	pub blob_fragment_size: u64,

	pub input: BTreeMap<SmartString<LazyCompact>, CopperData>,
}

impl PipelineJobContext<CopperData> for CopperContext {
	fn get_input(&self) -> &BTreeMap<SmartString<LazyCompact>, CopperData> {
		&self.input
	}
}
