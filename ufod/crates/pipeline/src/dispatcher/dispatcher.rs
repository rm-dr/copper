use itertools::Itertools;
use smartstring::{LazyCompact, SmartString};
use std::{collections::BTreeMap, marker::PhantomData};

use super::{NodeParameterSpec, NodeParameterValue};
use crate::{
	api::{PipelineData, PipelineJobContext, PipelineNode},
	nodes::input::{Input, INPUT_NODE_TYPE_NAME},
};

// This type must be send + sync, since we use this inside tokio's async runtime.
type InitNodeType<DataType, ContextType> = &'static (dyn Fn(
	// The job context to build this node with
	&ContextType,
	// This node's parameters
	&BTreeMap<SmartString<LazyCompact>, NodeParameterValue<DataType>>,
	// This node's name
	&str,
) -> Result<Box<dyn PipelineNode<DataType>>, ()>
              + Send
              + Sync);

/// A node type we've registered inside a [`NodeDispatcher`]
struct RegisteredNode<DataType: PipelineData, ContextType: PipelineJobContext<DataType>> {
	/// A method that constructs a new node of this type with the provided parameters.
	init_node: InitNodeType<DataType, ContextType>,

	/// The parameters this node takes
	parameters: Vec<NodeParameterSpec<DataType>>,
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
		x.register_node(INPUT_NODE_TYPE_NAME, vec![], &|ctx, params, name| {
			Ok(Box::new(Input::new(ctx, params, name)))
		})
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
		parameters: Vec<NodeParameterSpec<DataType>>,
		init_node: InitNodeType<DataType, ContextType>,
	) -> Result<(), ()> {
		if self.nodes.contains_key(type_name) {
			panic!()
		}

		if !parameters.iter().map(|x| &x.param_name).all_unique() {
			panic!()
		}

		self.nodes.insert(
			type_name.into(),
			RegisteredNode {
				init_node,
				parameters,
			},
		);

		return Ok(());
	}

	pub(crate) fn make_node(
		&self,
		context: &ContextType,
		node_type: &str,
		node_params: &BTreeMap<SmartString<LazyCompact>, NodeParameterValue<DataType>>,
		node_name: &str,
	) -> Result<Box<dyn PipelineNode<DataType>>, ()> {
		if let Some(node) = self.nodes.get(node_type) {
			return Ok((node.init_node)(context, node_params, node_name).unwrap());
		} else {
			panic!()
		}
	}
}
