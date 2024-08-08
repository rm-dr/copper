//! Pipeline node implementations

//#![warn(missing_docs)]
#![allow(clippy::new_without_default)]

mod helpers;
mod traits;

pub mod input;
pub mod output;
pub mod tags;
pub mod util;

pub mod errors;
pub mod nodeinstance;
pub mod nodetype;

use std::sync::{Arc, Mutex};
use ufo_metadb::api::MetaDb;

#[derive(Clone)]
pub struct UFOContext {
	pub dataset: Arc<Mutex<dyn MetaDb>>,

	/// How many fragments a blob channel can hold at once
	pub blob_channel_capacity: usize,

	/// The maximum size, in bytes, of a blob channel fragment
	pub blob_fragment_size: usize,
}
