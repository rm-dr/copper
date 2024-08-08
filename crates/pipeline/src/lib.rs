//#![warn(missing_docs)]

pub mod data;
pub mod errors;
mod graph;
pub mod node;
#[allow(clippy::module_inception)]
pub mod pipeline;
pub mod portspec;
pub mod runner;
pub mod syntax;
