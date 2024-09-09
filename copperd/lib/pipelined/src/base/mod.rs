//! Interfaces for defining pipeline nodes

mod data;
pub use data::*;

mod errors;
pub use errors::*;

mod node;
pub use node::*;

mod param;
pub use param::*;

mod labels;
pub use labels::*;

mod dispatcher;
pub use dispatcher::*;

mod internal_nodes;

pub const INPUT_NODE_TYPE_NAME: &str = "Input";
