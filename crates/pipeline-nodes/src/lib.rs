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

use ufo_db_blobstore::api::Blobstore;
use ufo_db_metastore::api::Metastore;

#[derive(Clone)]
pub struct UFOContext {
	pub metastore: Arc<dyn Metastore>,
	pub blobstore: Arc<dyn Blobstore>,

	/// The maximum size, in bytes, of a blob channel fragment
	pub blob_fragment_size: usize,
}
