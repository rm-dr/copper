use smartstring::{LazyCompact, SmartString};
use std::collections::BTreeMap;
use thiserror::Error;

use super::{Node, NodeParameterSpec};

pub trait NodeBuilder: Send + Sync {
	fn build<'ctx>(&self) -> Box<dyn Node<'ctx>>;
}

pub const INPUT_NODE_TYPE: &str = "Input";

/// An error we encounter when trying to register a node
#[derive(Debug, Error)]
pub enum RegisterNodeError {
	/// We tried to register a node with a type string that is already used
	#[error("A node with this name already exists")]
	AlreadyExists,
}

/// A node type we've registered inside a [`NodeDispatcher`]
struct RegisteredNode {
	/// A method that constructs a new node of this type with the provided parameters.
	builder: Box<dyn NodeBuilder>,

	/// The parameters this node takes
	_parameters: BTreeMap<SmartString<LazyCompact>, NodeParameterSpec>,
}

/// A factory struct that constructs pipeline nodes
pub struct NodeDispatcher {
	nodes: BTreeMap<SmartString<LazyCompact>, RegisteredNode>,
}

impl NodeDispatcher {
	/// Create a new [`NodeDispatcher`]
	pub fn new() -> Self {
		return Self {
			nodes: BTreeMap::new(),
		};
	}

	/// Register a new node type.
	///
	/// - `type_name` must be a new node type, we'll return an error if it already exists.
	/// - `init_node` is a method that constructs a new node of the given type with the provided parameters.
	pub fn register_node(
		&mut self,
		type_name: &str,
		parameters: BTreeMap<SmartString<LazyCompact>, NodeParameterSpec>,
		builder: Box<dyn NodeBuilder>,
	) -> Result<(), RegisterNodeError> {
		if self.nodes.contains_key(type_name) || type_name == INPUT_NODE_TYPE {
			return Err(RegisterNodeError::AlreadyExists);
		}

		self.nodes.insert(
			type_name.into(),
			RegisteredNode {
				builder,
				_parameters: parameters,
			},
		);

		return Ok(());
	}

	pub fn has_node(&self, node_name: &str) -> bool {
		return self.nodes.contains_key(node_name);
	}

	pub fn init_node<'ctx>(&self, node_type: &str) -> Option<Box<dyn Node<'ctx>>> {
		if let Some(node) = self.nodes.get(node_type) {
			return Some(node.builder.build());
		} else {
			return None;
		}
	}
}
