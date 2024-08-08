use smartstring::{LazyCompact, SmartString};
use std::{collections::BTreeMap, marker::PhantomData};

use super::{NodeParameterSpec, NodeParameterValue, RegisterNodeError};
use crate::{
	api::{InitNodeError, NodeInfo, PipelineData, PipelineJobContext, Node},
	nodes::input::{Input, InputInfo, INPUT_NODE_TYPE_NAME},
};

// This type must be send + sync, since we use this inside tokio's async runtime.
type NodeInitFnType<DataType, ContextType> = &'static (dyn Fn(
	// The job context to build this node with
	&ContextType,
	// This node's parameters
	&BTreeMap<SmartString<LazyCompact>, NodeParameterValue<DataType>>,
	// This node's name
	&str,
) -> Result<Box<dyn Node<DataType>>, InitNodeError>
              + Send
              + Sync);

// This type must be send + sync, since we use this inside tokio's async runtime.
type NodeInfoFnType<DataType, ContextType> = &'static (dyn Fn(
	// The job context to build this node with
	&ContextType,
	// This node's parameters
	&BTreeMap<SmartString<LazyCompact>, NodeParameterValue<DataType>>,
	// This node's name
	&str,
) -> Result<Box<dyn NodeInfo<DataType>>, InitNodeError>
              + Send
              + Sync);

/// A node type we've registered inside a [`NodeDispatcher`]
struct RegisteredNode<DataType: PipelineData, ContextType: PipelineJobContext<DataType>> {
	/// A method that constructs a new node of this type with the provided parameters.
	node_info: NodeInfoFnType<DataType, ContextType>,

	/// A method that constructs a new node of this type with the provided parameters.
	node_init: NodeInitFnType<DataType, ContextType>,

	/// The parameters this node takes
	_parameters: BTreeMap<SmartString<LazyCompact>, NodeParameterSpec<DataType>>,
}

/// A factory struct that constructs pipeline nodes
/// `ContextType` is per-job state that is passed to each node.
pub struct NodeDispatcher<DataType: PipelineData, ContextType: PipelineJobContext<DataType>> {
	_pa: PhantomData<DataType>,
	_pb: PhantomData<ContextType>,

	nodes: BTreeMap<SmartString<LazyCompact>, RegisteredNode<DataType, ContextType>>,
}

impl<DataType: PipelineData, ContextType: PipelineJobContext<DataType>>
	NodeDispatcher<DataType, ContextType>
{
	/// Create a new [`NodeDispatcher`]
	pub fn new() -> Self {
		let mut x = Self {
			_pa: PhantomData {},
			_pb: PhantomData {},
			nodes: BTreeMap::new(),
		};

		// Register internal nodes
		x.register_node(
			INPUT_NODE_TYPE_NAME,
			BTreeMap::new(),
			&|_ctx, params, name| Ok(Box::new(InputInfo::new(params, name)?)),
			&|ctx, params, name| Ok(Box::new(Input::new(ctx, params, name)?)),
		)
		.unwrap();

		x
	}

	/// Register a new node type.
	///
	/// - `type_name` must be a new node type, we'll return an error if it already exists.
	/// - `init_node` is a method that constructs a new node of the given type with the provided parameters.
	pub fn register_node(
		&mut self,
		type_name: &str,
		parameters: BTreeMap<SmartString<LazyCompact>, NodeParameterSpec<DataType>>,
		node_info: NodeInfoFnType<DataType, ContextType>,
		node_init: NodeInitFnType<DataType, ContextType>,
	) -> Result<(), RegisterNodeError> {
		if self.nodes.contains_key(type_name) {
			return Err(RegisterNodeError::AlreadyExists);
		}

		self.nodes.insert(
			type_name.into(),
			RegisteredNode {
				node_info,
				node_init,
				_parameters: parameters,
			},
		);

		return Ok(());
	}

	pub(crate) fn init_node(
		&self,
		context: &ContextType,
		node_type: &str,
		node_params: &BTreeMap<SmartString<LazyCompact>, NodeParameterValue<DataType>>,
		node_name: &str,
	) -> Result<Option<Box<dyn Node<DataType>>>, InitNodeError> {
		if let Some(node) = self.nodes.get(node_type) {
			return Ok(Some((node.node_init)(context, node_params, node_name)?));
		} else {
			return Ok(None);
		}
	}

	pub(crate) fn node_info(
		&self,
		context: &ContextType,
		node_type: &str,
		node_params: &BTreeMap<SmartString<LazyCompact>, NodeParameterValue<DataType>>,
		node_name: &str,
	) -> Result<Option<Box<dyn NodeInfo<DataType>>>, InitNodeError> {
		if let Some(node) = self.nodes.get(node_type) {
			return Ok(Some((node.node_info)(context, node_params, node_name)?));
		} else {
			return Ok(None);
		}
	}
}
