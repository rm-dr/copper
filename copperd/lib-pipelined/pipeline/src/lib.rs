//! Fast, flexible, parallel data processing pipelines.

#![warn(missing_docs)]

pub(crate) mod nodes;

pub mod base;
pub mod dispatcher;
pub mod labels;
#[allow(clippy::module_inception)]
pub mod pipeline;
pub mod runner;
