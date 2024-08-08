//! Pipeline node implementations

//#![warn(missing_docs)]
#![allow(clippy::new_without_default)]

pub mod data;
mod helpers;
mod traits;

pub mod database;
pub mod input;
pub mod tags;
pub mod util;

pub mod errors;
pub mod nodeinstance;
pub mod nodetype;

use std::sync::Arc;
use ufo_ds_impl::local::LocalDataset;

#[derive(Clone)]
pub struct UFOContext {
	// Hard-code LocalDataset for now,
	// TODO: this should be some form of "generic dataset" later.
	//
	// Maybe don't provide a dataset, but a way to *get* datasets?
	// (maindb ref?)
	pub dataset: Arc<LocalDataset>,

	/// The maximum size, in bytes, of a blob channel fragment
	pub blob_fragment_size: usize,
}
