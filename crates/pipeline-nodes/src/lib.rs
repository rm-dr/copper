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

use std::sync::{Arc, Mutex};
use ufo_database::api::UFODatabase;

#[derive(Clone)]
pub struct UFOContext {
	pub database: Arc<Mutex<dyn UFODatabase>>,

	/// The maximum size, in bytes, of a blob channel fragment
	pub blob_fragment_size: usize,
}
