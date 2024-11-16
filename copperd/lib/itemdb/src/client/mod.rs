//! This modules contains Copper's itemdb client

#[allow(clippy::module_inception)]
mod client;
pub use client::*;

pub mod errors;

pub(crate) mod migrate;
