use copper_pipelined::base::{
	Node, NodeDispatcher, NodeId, NodeOutput, NodeParameterValue, PipelineData, PipelineJobContext,
	PortName, RunNodeError, ThisNodeInfo, INPUT_NODE_TYPE,
};
use copper_util::graph::{finalized::FinalizedGraph, graph::Graph, util::GraphNodeIdx};
use smartstring::{LazyCompact, SmartString};
use std::{
	collections::{BTreeMap, HashMap},
	error::Error,
	fmt::{Debug, Display},
	marker::PhantomData,
	sync::Arc,
};
use tokio::{
	sync::mpsc::{self, error::TryRecvError},
	task::{JoinError, JoinSet},
};
use tracing::{debug, trace};

use super::json::PipelineJson;
use crate::config::ASYNC_POLL_AWAIT_MS;

//
// MARK: Errors
//

/// An error we encounter when a pipeline spec is invalid
#[derive(Debug)]
pub enum PipelineBuildError {
	/// An edge references a node, but it doesn't exist
	NoNode {
		/// The edge that references an invalid node
		edge_id: SmartString<LazyCompact>,

		/// The node id that doesn't exist
		invalid_node_id: NodeId,
	},

	/// We found a node with an invalid type
	BadNodeType { bad_type: SmartString<LazyCompact> },

	/// This pipeline has a cycle and is thus invalid
	HasCycle,

	/// We expected an input, but it wasn't provided
	MissingInput { input: SmartString<LazyCompact> },

	/// An input node wasn't specified properly
	InvalidInputNode { node: NodeId },
}

impl Display for PipelineBuildError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::NoNode {
				edge_id,
				invalid_node_id,
			} => {
				writeln!(
					f,
					"edge `{edge_id}` references a node `{invalid_node_id}` that doesn't exist"
				)
			}

			Self::BadNodeType { bad_type } => {
				writeln!(f, "invalid node type `{bad_type}`")
			}

			Self::HasCycle => {
				writeln!(f, "this pipeline has a cycle")
			}

			Self::MissingInput { input } => {
				writeln!(f, "missing pipeline input `{input}`")
			}

			Self::InvalidInputNode { node } => {
				writeln!(f, "Input node `{node}` is invalid")
			}
		}
	}
}

impl Error for PipelineBuildError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		None
	}
}

//
// MARK: Helper structs
//

struct NodeResult<DataType: PipelineData> {
	result: Result<(), RunNodeError<DataType>>,
	node_id: NodeId,
	node_idx: GraphNodeIdx,
}

enum NodeState<DataType: PipelineData, ContextType: PipelineJobContext> {
	// Store this node's instance here, so we can take ownership
	// of it when we run.
	NotStarted {
		instance: Box<dyn Node<DataType, ContextType>>,
	},
	Running,
	Done,
}

impl<DataType: PipelineData, ContextType: PipelineJobContext> Debug
	for NodeState<DataType, ContextType>
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::NotStarted { .. } => write!(f, "NotStarted"),
			Self::Done => write!(f, "Done"),
			Self::Running => write!(f, "Running"),
		}
	}
}

impl<DataType: PipelineData, ContextType: PipelineJobContext> NodeState<DataType, ContextType> {
	fn has_been_started(&self) -> bool {
		return matches!(self, Self::Running | Self::Done);
	}

	fn is_running(&self) -> bool {
		return matches!(self, Self::Running);
	}

	fn is_done(&self) -> bool {
		return matches!(self, Self::Done);
	}

	/// Turn `Self::NotStarted` into `Self::Running` and return `instance`.
	/// Returns `None` otherwise.
	fn start(&mut self) -> Option<Box<dyn Node<DataType, ContextType>>> {
		match self {
			Self::NotStarted { .. } => {
				let x = std::mem::replace(self, Self::Running);
				match x {
					Self::NotStarted { instance } => Some(instance),
					_ => unreachable!(),
				}
			}
			_ => None,
		}
	}
}

struct NodeSpec<DataType: PipelineData, ContextType: PipelineJobContext> {
	/// The node's id
	pub id: NodeId,

	/// This node's type
	pub node_type: SmartString<LazyCompact>,

	/// This node's parameters.
	pub node_params: BTreeMap<SmartString<LazyCompact>, NodeParameterValue>,

	/// This node's state
	pub state: NodeState<DataType, ContextType>,
}

// We need to do this ourselves, since the ContextType generic
// confuses #[derive(Debug)].
impl<DataType: PipelineData, ContextType: PipelineJobContext> Debug
	for NodeSpec<DataType, ContextType>
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("NodeSpec")
			.field("id", &self.id)
			.field("node_type", &self.node_type)
			.field("node_params", &self.node_params)
			.field("state", &self.state)
			.finish()
	}
}

#[derive(Debug)]
enum EdgeDataContainer<DataType: PipelineData> {
	Unset,
	Some(Option<DataType>),
	Consumed,
}

impl<DataType: PipelineData> EdgeDataContainer<DataType> {
	/// Takes the value out of this container, leaving [`EdgeDataContainer::Consumed`] in its place.
	/// Does nothing and returns None if this isn't [`EdgeDataContainer::Some`].
	pub fn take(&mut self) -> Option<Option<DataType>> {
		match self {
			Self::Some(_) => {
				let x = std::mem::replace(self, Self::Consumed);

				return Some(match x {
					Self::Some(x) => x,
					_ => unreachable!(),
				});
			}
			_ => None,
		}
	}
}

#[derive(Debug)]
struct EdgeSpec<DataType: PipelineData> {
	source_port: PortName,
	target_port: PortName,
	data: EdgeDataContainer<DataType>,
}

//
// MARK: PipelineSpec
//

/// A pipeline specification built from [`PipelineJson`].
///
/// This is the second step in our pipeline processing workflow.
/// Any [`PipelineJson`] that builds into a PipelineSpec successfully
/// should be runnable (but may encounter run-time errors)
#[derive(Debug)]
pub struct PipelineJob<DataType: PipelineData, ContextType: PipelineJobContext> {
	_pa: PhantomData<DataType>,
	_pb: PhantomData<ContextType>,
	pub job_id: SmartString<LazyCompact>,

	graph: FinalizedGraph<NodeSpec<DataType, ContextType>, EdgeSpec<DataType>>,
}

impl<DataType: PipelineData, ContextType: PipelineJobContext> PipelineJob<DataType, ContextType> {
	pub fn new(
		dispatcher: &NodeDispatcher<DataType, ContextType>,
		job_id: &str,
		input: BTreeMap<SmartString<LazyCompact>, DataType>,
		json: &PipelineJson,
	) -> Result<Self, PipelineBuildError> {
		return Ok(Self {
			_pa: PhantomData {},
			_pb: PhantomData {},
			job_id: job_id.into(),
			graph: Self::build(dispatcher, job_id, json, input)?,
		});
	}

	//
	// MARK: Build
	//

	/// Build a pipeline from its deserialized form
	fn build(
		dispatcher: &NodeDispatcher<DataType, ContextType>,
		job_id: &str,
		json: &PipelineJson,
		input: BTreeMap<SmartString<LazyCompact>, DataType>,
	) -> Result<
		FinalizedGraph<NodeSpec<DataType, ContextType>, EdgeSpec<DataType>>,
		PipelineBuildError,
	> {
		trace!(message = "Building pipeline graph", job_id);

		// The graph that stores this pipeline
		let mut graph = Graph::new();
		// Maps node ids (from JSON) to node indices in `graph`
		let mut node_id_map = HashMap::new();

		// Create all nodes in the graph
		trace!(message = "Making nodes", job_id);
		for (node_id, node_spec) in &json.nodes {
			if node_spec.node_type == INPUT_NODE_TYPE {
				let n = graph.add_node(NodeSpec {
					id: node_id.clone(),
					// Input nodes are never run, start them as "done".
					// They are filled in at the end of this method.
					state: NodeState::Done,
					node_params: node_spec.params.clone(),
					node_type: node_spec.node_type.clone(),
				});
				node_id_map.insert(node_id.clone(), n);
			} else {
				let node_instance = dispatcher.init_node(&node_spec.node_type);
				if node_instance.is_none() {
					return Err(PipelineBuildError::BadNodeType {
						bad_type: node_spec.node_type.clone(),
					});
				}

				let n = graph.add_node(NodeSpec {
					id: node_id.clone(),
					state: NodeState::NotStarted {
						instance: node_instance.unwrap(),
					},
					node_params: node_spec.params.clone(),
					node_type: node_spec.node_type.clone(),
				});

				node_id_map.insert(node_id.clone(), n);
			}
		}

		// Make sure all edges are valid and create them in the graph.
		//
		// We do not check if ports exist & have matching types here,
		// since not all nodes know their ports at build time.
		trace!(message = "Making edges", job_id);
		for (edge_id, edge_spec) in json.edges.iter() {
			// These should never fail
			let source_node_idx = node_id_map.get(&edge_spec.source.node);
			if source_node_idx.is_none() {
				return Err(PipelineBuildError::NoNode {
					edge_id: edge_id.clone(),
					invalid_node_id: edge_spec.source.node.clone(),
				});
			}

			let target_node_idx = node_id_map.get(&edge_spec.target.node);
			if target_node_idx.is_none() {
				return Err(PipelineBuildError::NoNode {
					edge_id: edge_id.clone(),
					invalid_node_id: edge_spec.target.node.clone(),
				});
			}

			// Create the edge
			graph.add_edge(
				*source_node_idx.unwrap(),
				*target_node_idx.unwrap(),
				EdgeSpec {
					source_port: edge_spec.source.port.clone(),
					target_port: edge_spec.target.port.clone(),
					data: EdgeDataContainer::Unset,
				},
			);
		}

		trace!(message = "Looking for cycles", job_id);
		// Make sure our graph doesn't have any cycles
		if graph.has_cycle() {
			return Err(PipelineBuildError::HasCycle);
		}

		let mut finalized_graph = graph.finalize();

		trace!(message = "Filling edges connected to input nodes", job_id);
		// Find input nodes...
		let input_nodes: Vec<GraphNodeIdx> = finalized_graph
			.iter_nodes_idx()
			.filter_map(|(idx, node)| {
				if node.node_type == INPUT_NODE_TYPE {
					return Some(idx);
				} else {
					return None;
				}
			})
			.collect();

		// ...and "run" them.
		for idx in input_nodes {
			let node = finalized_graph.get_node(idx).unwrap();
			let input_param =
				node.node_params
					.get("input_name")
					.ok_or(PipelineBuildError::InvalidInputNode {
						node: node.id.clone(),
					})?;
			let input_name: SmartString<LazyCompact> = match input_param {
				NodeParameterValue::String(s) => s.clone(),
				_ => {
					return Err(PipelineBuildError::InvalidInputNode {
						node: node.id.clone(),
					})
				}
			};

			if let Some(i_val) = input.get(&input_name) {
				let edges = Vec::from(finalized_graph.edges_starting_at(idx).unwrap());

				for edge_idx in edges {
					let (_, _, edge) = finalized_graph.get_edge_mut(edge_idx).unwrap();
					edge.data = EdgeDataContainer::Some(Some(i_val.clone()))
				}

				finalized_graph.get_node_mut(idx).unwrap().state = NodeState::Done;
			} else {
				return Err(PipelineBuildError::MissingInput { input: input_name });
			}
		}

		trace!(message = "Pipeline graph is ready", job_id);
		return Ok(finalized_graph);
	}

	//
	// MARK: Run
	//

	pub async fn run(mut self, context: Arc<ContextType>) -> Result<(), RunNodeError<DataType>> {
		trace!(
			message = "Running job",
			job_id = ?self.job_id,
			graph = ?self.graph
		);

		let all_nodes: Vec<GraphNodeIdx> =
			self.graph.iter_nodes_idx().map(|(idx, _)| idx).collect();

		let mut tasks = JoinSet::new();
		let (tx, mut rx) = mpsc::channel::<NodeOutput<DataType>>(30);

		// The below loop hits no awaits until something interesting happens
		// (node finished, input received, etc)
		//
		// This is a problem, because this loop will block the async runtime,
		// which prevents interesting things from happening.
		//
		// We've thus added `yield_now` to this loop.
		loop {
			//
			// Start nodes
			//
			for node_idx in &all_nodes {
				let node_idx = *node_idx;

				// Never run nodes twice
				if self
					.graph
					.get_node(node_idx)
					.unwrap()
					.state
					.has_been_started()
				{
					continue;
				}

				let can_be_run =
					self.graph
						.edges_ending_at(node_idx)
						.unwrap()
						.iter()
						.all(|edge_idx| {
							let (_, _, edge) = self.graph.get_edge(*edge_idx).unwrap();
							match edge.data {
								EdgeDataContainer::Unset => false,
								EdgeDataContainer::Some(_) => true,
								EdgeDataContainer::Consumed => unreachable!(),
							}
						});

				if !can_be_run {
					continue;
				}

				// Take all inputs
				let node_run_input: BTreeMap<PortName, Option<DataType>> = {
					let input_edges = Vec::from(self.graph.edges_ending_at(node_idx).unwrap());
					input_edges
						.into_iter()
						.map(|edge_idx| {
							let (_, _, edge) = self.graph.get_edge_mut(edge_idx).unwrap();
							(edge.target_port.clone(), edge.data.take().unwrap())
						})
						.collect()
				};

				// Borrow again as mutable
				let node = self.graph.get_node_mut(node_idx).unwrap();
				let ctx = context.clone();
				let params = node.node_params.clone();
				let node_id = node.id.clone();
				let node_type = node.node_type.clone();
				let tx = tx.clone();
				let node_inst = node.state.start().unwrap();
				let job_id = self.job_id.clone();

				debug!(
					message = "Starting node",
					node_type = ?node_type,
					node_id = ?node_id,
					job_id = ?job_id,
					tasks = tasks.len(),
				);

				tasks.spawn(async move {
					// This log helps detect a blocked async runtime
					// (i.e, tokio is stuck if you see "Starting node" and not "Started node")
					trace!(
						message = "Started node",
						node_type = ?node_type,
						node_id = ?node_id,
						job_id = ?job_id,
					);

					let result = node_inst
						.run(
							&ctx,
							ThisNodeInfo {
								id: node_id.clone(),
								idx: node_idx,
								node_type: node_type.clone(),
							},
							params,
							node_run_input,
							tx,
						)
						.await;

					NodeResult {
						result,
						node_id,
						node_idx,
					}
				});
			}

			tokio::task::yield_now().await;

			//
			// Process node output
			//
			loop {
				match rx.try_recv() {
					Err(TryRecvError::Empty) => {
						break;
					}

					Err(TryRecvError::Disconnected) => {
						panic!()
					}

					Ok(x) => self.process_node_output(x)?,
				}
			}

			tokio::task::yield_now().await;

			//
			// Process finished tasks
			//
			while let Some(res) = tasks.try_join_next() {
				self.process_finished_node(res)?
			}

			//
			// Exit if we have no more tasks to run
			// (i.e, if there are no pending tasks AND all nodes are done)
			//
			if tasks.is_empty()
				&& all_nodes
					.iter()
					.all(|x| self.graph.get_node(*x).unwrap().state.is_done())
			{
				break;
			}

			tokio::time::sleep(std::time::Duration::from_millis(ASYNC_POLL_AWAIT_MS)).await;
		}

		return Ok(());
	}

	//
	// MARK: Process task results
	//
	fn process_node_output(
		&mut self,
		out: NodeOutput<DataType>,
	) -> Result<(), RunNodeError<DataType>> {
		trace!(
			message = "Processing node output",
			node_type= ?out.node.node_type,
			node_id = ?out.node.id,
			job_id = ?self.job_id,
		);

		// Send output to edges
		for (from_node, _to_node, edge) in self.graph.iter_edges_mut() {
			if from_node != out.node.idx || edge.source_port != out.port {
				continue;
			}

			if !matches!(edge.data, EdgeDataContainer::Unset) {
				return Err(RunNodeError::OutputPortSetTwice {
					node_id: out.node.id,
					node_type: out.node.node_type,
					port: edge.source_port.clone(),
				});
			}

			edge.data = EdgeDataContainer::Some(out.data.clone());
		}

		return Ok(());
	}

	fn process_finished_node(
		&mut self,
		res: Result<NodeResult<DataType>, JoinError>,
	) -> Result<(), RunNodeError<DataType>> {
		let res = res?;
		let node = self.graph.get_node_mut(res.node_idx).unwrap();

		assert!(
			node.state.is_running(),
			"Expected node to be running. node: {node:?}"
		);

		node.state = NodeState::Done;

		if let Err(err) = res.result {
			debug!(
				message = "Node run failed",
				node_id = ?res.node_id,
				job_id = ?self.job_id,
				error = format!("{err:?}")
			);
			return Err(err);
		} else {
			debug!(
				message = "Node finished without error",
				node_id = ?res.node_id,
				job_id = ?self.job_id,
			);
		}

		// Make sure all edges starting at this node got output
		for (from_node, _to_node, edge) in self.graph.iter_edges_mut() {
			if from_node != res.node_idx {
				continue;
			}

			if matches!(edge.data, EdgeDataContainer::Unset) {
				return Err(RunNodeError::UnrecognizedOutput {
					port: edge.source_port.clone(),
				});
			}
		}

		return Ok(());
	}
}
