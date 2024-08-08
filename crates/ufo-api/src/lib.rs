use serde::{Deserialize, Serialize};
use smartstring::{LazyCompact, SmartString};

pub mod data;
pub mod pipeline;
pub mod runner;
pub mod upload;

#[derive(Deserialize, Serialize, Debug)]
pub struct ServerStatus {
	pub version: SmartString<LazyCompact>,
}
