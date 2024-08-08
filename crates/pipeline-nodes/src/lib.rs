//#![warn(missing_docs)]

use std::sync::Mutex;
use ufo_storage::sqlite::dataset::SQLiteDataset;

pub mod data;
pub mod input;
pub mod output;
pub mod tags;
pub mod util;

pub mod nodeinstance;
pub mod nodetype;

pub struct UFOContext {
	// TODO: trait object
	pub dataset: Mutex<SQLiteDataset>,
}
