use copper_pipelined::base::{
	NodeDispatcher, NodeId, NodeOutput, NodeParameterValue, PipelineData, PipelineJobContext,
	PortName, RunNodeError, ThisNodeInfo,
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
	task::JoinSet,
};
use tracing::{debug, trace};

use super::json::PipelineJson;

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

	/// This pipeline has a cycle and is thus invalid
	HasCycle,
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

			Self::HasCycle => {
				writeln!(f, "this pipeline has a cycle")
			}
		}
	}
}

impl Error for PipelineBuildError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			_ => None,
		}
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

#[derive(Debug, PartialEq, Eq)]
enum NodeState {
	NotStarted,
	Running,
	Done,
}

#[derive(Debug)]
struct NodeSpec<DataType: PipelineData> {
	/// The node's id
	pub id: NodeId,

	/// This node's type
	pub node_type: SmartString<LazyCompact>,

	/// This node's parameters.
	pub node_params: BTreeMap<SmartString<LazyCompact>, NodeParameterValue<DataType>>,

	/// This node's state
	pub state: NodeState,
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

	graph: FinalizedGraph<NodeSpec<DataType>, EdgeSpec<DataType>>,
}

impl<DataType: PipelineData, ContextType: PipelineJobContext> PipelineJob<DataType, ContextType> {
	pub fn new(
		job_id: &str,
		input: BTreeMap<SmartString<LazyCompact>, DataType>,
		json: &PipelineJson<DataType>,
	) -> Result<Self, PipelineBuildError> {
		let mut graph = Self::build(job_id, json)?;

		// Find input nodes...
		let input_nodes: Vec<(GraphNodeIdx, NodeId)> = graph
			.iter_nodes_idx()
			.filter_map(|(idx, node)| {
				// TODO: const & reserve in dispatcher
				if node.node_type != "Input" {
					return None;
				} else {
					return Some((idx, node.id.clone()));
				}
			})
			.collect();

		// ...and "run" them.
		for (idx, node_id) in input_nodes {
			if let Some(i_val) = input.get(node_id.id()) {
				// TODO: no clone
				let edges = Vec::from(graph.edges_starting_at(idx));

				for edge_idx in edges {
					let (_, _, edge) = graph.get_edge_mut(edge_idx);
					edge.data = EdgeDataContainer::Some(Some(i_val.clone()))
				}

				graph.get_node_mut(idx).state = NodeState::Done;
			} else {
				panic!("Missing input")
			}
		}

		return Ok(Self {
			_pa: PhantomData {},
			_pb: PhantomData {},
			job_id: job_id.into(),
			graph,
		});
	}

	//
	// MARK: Build
	//

	/// Build a pipeline from its deserialized form
	fn build(
		job_id: &str,
		json: &PipelineJson<DataType>,
	) -> Result<FinalizedGraph<NodeSpec<DataType>, EdgeSpec<DataType>>, PipelineBuildError> {
		debug!(message = "Building pipeline graph", job_id);

		// The graph that stores this pipeline
		let mut graph = Graph::new();
		// Maps node ids (from JSON) to node indices in `graph`
		let mut node_id_map = HashMap::new();

		// Create all nodes in the graph
		trace!(message = "Making nodes", job_id);
		for (node_id, node_spec) in &json.nodes {
			let n = graph.add_node(NodeSpec {
				id: node_id.clone(),
				state: NodeState::NotStarted,
				node_params: node_spec.params.clone(),
				node_type: node_spec.node_type.clone(),
			});

			node_id_map.insert(node_id.clone(), n);
		}

		// Make sure all edges are valid and create them in the graph.
		//
		// We do not check if ports exist & have matching types here,
		// since not all nodes know their ports at build time.
		trace!(message = "Making  edges", job_id);
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

		trace!(message = "Pipeline graph is ready", job_id);
		return Ok(graph.finalize());
	}

	//
	// MARK: Run
	//

	pub async fn run(
		mut self,
		context: Arc<ContextType>,
		dispatcher: &NodeDispatcher<DataType, ContextType>,
	) -> Result<(), RunNodeError<DataType>> {
		trace!(
			message = "Running job",
			job_id = ?self.job_id,
			graph = ?self.graph
		);

		let all_nodes: Vec<GraphNodeIdx> =
			self.graph.iter_nodes_idx().map(|(idx, _)| idx).collect();

		let mut tasks = JoinSet::new();
		let (tx, mut rx) = mpsc::channel::<NodeOutput<DataType>>(10);

		loop {
			//
			// Start nodes
			//
			for node_idx in &all_nodes {
				let node_idx = *node_idx;

				// Never run nodes twice
				if self.graph.get_node(node_idx).state != NodeState::NotStarted {
					continue;
				}

				let can_be_run = self.graph.edges_ending_at(node_idx).iter().all(|edge_idx| {
					let (_, _, edge) = self.graph.get_edge(*edge_idx);
					match edge.data {
						EdgeDataContainer::Unset => false,
						EdgeDataContainer::Some(_) => true,
						EdgeDataContainer::Consumed => unreachable!(),
					}
				});

				if !can_be_run {
					continue;
				}

				// Initialize and run
				let node = self.graph.get_node(node_idx);
				trace!(
					message = "Running node",
					node_type= ?node.node_type,
					node_id = ?node.id,
					job_id = ?self.job_id,
				);

				// Take all inputs
				let node_run_input: BTreeMap<PortName, Option<DataType>> = {
					let input_edges = Vec::from(self.graph.edges_ending_at(node_idx));
					input_edges
						.into_iter()
						.filter_map(|edge_idx| {
							let (_, _, edge) = self.graph.get_edge_mut(edge_idx);
							Some((edge.target_port.clone(), edge.data.take().unwrap()))
						})
						.collect()
				};

				// Borrow again as mutable
				let node = self.graph.get_node_mut(node_idx);
				node.state = NodeState::Running;

				let ctx = context.clone();
				let params = node.node_params.clone();
				let node_id = node.id.clone();
				let node_type = node.node_type.clone();
				let tx = tx.clone();

				// This should never fail, node types are checked at build time
				let node_inst = dispatcher.init_node(&node.node_type).unwrap();
				tasks.spawn(async move {
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

					Ok(x) => self.process_node_output(x).await?,
				}
			}

			//
			// Process finished tasks
			//
			while let Some(res) = tasks.try_join_next() {
				let res = res?;

				self.graph.get_node_mut(res.node_idx).state = NodeState::Done;

				if let Err(err) = res.result {
					trace!(
						message = "Node run failed",
						node_id = ?res.node_id,
						job_id = ?self.job_id,
						error = format!("{err:?}")
					);
					return Err(err);
				} else {
					trace!(
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
			}

			//
			// Exit if we have no more tasks to run
			// (i.e, if there are no pending tasks AND all nodes are done)
			//
			if tasks.is_empty()
				&& all_nodes
					.iter()
					.all(|x| self.graph.get_node(*x).state == NodeState::Done)
			{
				break;
			}
		}

		debug!(message = "Job finished", job_id = ?self.job_id);

		return Ok(());
	}

	//
	// MARK: Process task result
	//
	async fn process_node_output(
		&mut self,
		out: NodeOutput<DataType>,
	) -> Result<(), RunNodeError<DataType>> {
		let node = self.graph.get_node_mut(out.node.idx);
		node.state = NodeState::Done;

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
}
