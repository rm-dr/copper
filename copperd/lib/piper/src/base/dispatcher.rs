use smartstring::{LazyCompact, SmartString};
use std::{collections::BTreeMap, marker::PhantomData};
use thiserror::Error;

use super::{Node, NodeParameterSpec, PipelineData, PipelineJobContext, PipelineJobResult};

// This type must be send + sync, since we use this inside tokio's async runtime.
type NodeInitFnType<ResultType, DataType, ContextType> =
	&'static (dyn Fn() -> Box<dyn Node<ResultType, DataType, ContextType>> + Send + Sync);

pub const INPUT_NODE_TYPE: &str = "Input";

/// An error we encounter when trying to register a node
#[derive(Debug, Error)]
pub enum RegisterNodeError {
	/// We tried to register a node with a type string that is already used
	#[error("A node with this name already exists")]
	AlreadyExists,
}

/// A node type we've registered inside a [`NodeDispatcher`]
struct RegisteredNode<
	ResultType: PipelineJobResult,
	DataType: PipelineData,
	ContextType: PipelineJobContext<DataType, ResultType>,
> {
	_p: PhantomData<ResultType>,

	/// A method that constructs a new node of this type with the provided parameters.
	node_init: NodeInitFnType<ResultType, DataType, ContextType>,

	/// The parameters this node takes
	_parameters: BTreeMap<SmartString<LazyCompact>, NodeParameterSpec>,
}

/// A factory struct that constructs pipeline nodes
pub struct NodeDispatcher<
	ResultType: PipelineJobResult,
	DataType: PipelineData,
	ContextType: PipelineJobContext<DataType, ResultType>,
> {
	_pa: PhantomData<DataType>,
	nodes: BTreeMap<SmartString<LazyCompact>, RegisteredNode<ResultType, DataType, ContextType>>,
}

impl<
		ResultType: PipelineJobResult,
		DataType: PipelineData,
		ContextType: PipelineJobContext<DataType, ResultType>,
	> NodeDispatcher<ResultType, DataType, ContextType>
{
	/// Create a new [`NodeDispatcher`]
	pub fn new() -> Self {
		return Self {
			_pa: PhantomData {},
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
		node_init: NodeInitFnType<ResultType, DataType, ContextType>,
	) -> Result<(), RegisterNodeError> {
		if self.nodes.contains_key(type_name) || type_name == INPUT_NODE_TYPE {
			return Err(RegisterNodeError::AlreadyExists);
		}

		self.nodes.insert(
			type_name.into(),
			RegisteredNode {
				_p: PhantomData {},
				node_init,
				_parameters: parameters,
			},
		);

		return Ok(());
	}

	pub fn has_node(&self, node_name: &str) -> bool {
		return self.nodes.contains_key(node_name);
	}

	pub fn init_node(
		&self,
		node_type: &str,
	) -> Option<Box<dyn Node<ResultType, DataType, ContextType>>> {
		if let Some(node) = self.nodes.get(node_type) {
			return Some((node.node_init)());
		} else {
			return None;
		}
	}
}
