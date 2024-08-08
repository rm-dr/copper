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

use nodetype::UFONodeType;
use std::sync::Arc;
use ufo_ds_core::api::Dataset;

#[derive(Clone)]
pub struct UFOContext {
	pub dataset: Arc<dyn Dataset<UFONodeType>>,

	/// The maximum size, in bytes, of a blob channel fragment
	pub blob_fragment_size: usize,
}
