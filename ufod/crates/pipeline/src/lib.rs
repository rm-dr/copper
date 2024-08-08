//! Fast, flexible, parallel data processing pipelines.

#![warn(missing_docs)]

mod graph;

pub(crate) mod nodes;

pub mod api;
pub mod dispatcher;
pub mod labels;
#[allow(clippy::module_inception)]
pub mod pipeline;
pub mod runner;
