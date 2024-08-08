//! Pipeline node implementations

//#![warn(missing_docs)]
#![allow(clippy::new_without_default)]

mod helpers;

pub mod input;
pub mod output;
pub mod tags;
pub mod util;

pub mod nodeinstance;
pub mod nodetype;

use std::sync::{Arc, Mutex};
use ufo_storage::api::Dataset;

#[derive(Clone)]
pub struct UFOContext {
	pub dataset: Arc<Mutex<dyn Dataset>>,
}
