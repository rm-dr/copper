//#![warn(missing_docs)]

use std::sync::{Arc, Mutex};
use ufo_storage::api::Dataset;

pub mod input;
pub mod output;
pub mod tags;
pub mod util;

pub mod nodeinstance;
pub mod nodetype;

#[derive(Clone)]
pub struct UFOContext {
	pub dataset: Arc<Mutex<dyn Dataset>>,
}
