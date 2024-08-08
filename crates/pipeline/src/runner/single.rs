use crossbeam::channel::{unbounded, Receiver, Sender};
use std::{
	marker::PhantomData,
	sync::{Arc, Mutex},
};
use threadpool::ThreadPool;

use super::util::{EdgeValue, NodeRunState};
use crate::{
	api::{PipelineData, PipelineNode, PipelineNodeState, PipelineNodeStub},
	errors::PipelineError,
	graph::util::GraphNodeIdx,
	labels::PipelineNodeLabel,
	pipeline::Pipeline,
	SDataType, SNodeType,
};

/// The state of a [`PipelineSingleRunner`]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SingleRunnerState {
	/// Nodes are running, not done yet
	Running,

	/// Pipeline is done, this runner may be dropped
	Done,
}

/// An instance of a single running pipeline
pub(super) struct PipelineSingleRunner<StubType: PipelineNodeStub> {
	_p: PhantomData<StubType>,

	/// The pipeline we're running
	pipeline: Arc<Pipeline<StubType>>,

	/// The context for this pipeline
	context: Arc<<StubType::NodeType as PipelineNode>::NodeContext>,

	/// The inputs that were provided to this pipeline
	pipeline_inputs: Vec<SDataType<StubType>>,

	/// Mutable instances of each node in this pipeline
	node_instances: Vec<(PipelineNodeLabel, Arc<Mutex<SNodeType<StubType>>>)>,

	/// Each node's status
	/// (indices match `node_instances`)
	node_status: Vec<NodeRunState>,

	/// The value each edge in this pipeline carries
	edge_values: Vec<EdgeValue<SDataType<StubType>>>,

	/// A threadpool of node runners
	pool: ThreadPool,

	/// A copy of this is given to every node.
	/// Nodes send outputs here.
	send_data: Sender<(
		// The node that sent this message
		GraphNodeIdx,
		// The port index of this output
		usize,
		// The data that output produced
		SDataType<StubType>,
	)>,

	/// A receiver for node output data
	receive_data: Receiver<(
		// The node that sent this message
		GraphNodeIdx,
		// The port index of this output
		usize,
		// The data that output produced
		SDataType<StubType>,
	)>,

	/// A message is sent here whenever a node finishes running.
	send_status: Sender<(
		// The node that sent this status
		GraphNodeIdx,
		// The status that was sent
		Result<PipelineNodeState, PipelineError>,
	)>,

	/// A receiver for node status messages
	receive_status: Receiver<(
		// The node that sent this status
		GraphNodeIdx,
		// The status that was sent
		Result<PipelineNodeState, PipelineError>,
	)>,
}

impl<StubType: PipelineNodeStub> PipelineSingleRunner<StubType> {
	/// Make a new [`PipelineSingleRunner`]
	pub fn new(
		node_runners: usize,
		context: Arc<<StubType::NodeType as PipelineNode>::NodeContext>,
		pipeline: Arc<Pipeline<StubType>>,
		pipeline_inputs: Vec<SDataType<StubType>>,
	) -> Self {
		assert!(
			pipeline_inputs.len()
				== pipeline
					.graph
					.get_node(pipeline.input_node_idx)
					.1
					.n_inputs(&context)
		);

		let node_instances = pipeline
			.graph
			.iter_nodes()
			.map(|(name, x)| {
				(
					name.clone(),
					Arc::new(Mutex::new(x.build(&context, name.into()))),
				)
			})
			.collect::<Vec<_>>();

		// The data inside each edge.
		// We consume node data once it is read so that unneeded memory may be freed.
		let edge_values = {
			pipeline
				.graph
				.iter_edges()
				.map(|_| EdgeValue::Uninitialized)
				.collect::<Vec<_>>()
		};

		// Keep track of nodes we have already run.
		// We already initialized all input edges, so mark that node `true`.
		let node_status = pipeline
			.graph
			.iter_nodes_idx()
			.map(|_| NodeRunState::NotRunning(PipelineNodeState::NotStarted))
			.collect::<Vec<_>>();

		// Threadpool we'll use to run nodes
		let pool = threadpool::Builder::new()
			.num_threads(node_runners)
			.thread_name("Pipeline node runner".into())
			.build();

		// Channel for node data. Nodes send their outputs here once they are ready.
		//
		// Contents are (node index, port index, data)
		#[allow(clippy::type_complexity)]
		let (send_data, receive_data): (
			Sender<(GraphNodeIdx, usize, SDataType<StubType>)>,
			Receiver<(GraphNodeIdx, usize, SDataType<StubType>)>,
		) = unbounded();

		// Channel for node status. A node's return status is sent here when it finishes.
		//
		// Contents are (node index, result of `node.run()`)
		#[allow(clippy::type_complexity)]
		let (send_status, receive_status): (
			Sender<(GraphNodeIdx, Result<PipelineNodeState, PipelineError>)>,
			Receiver<(GraphNodeIdx, Result<PipelineNodeState, PipelineError>)>,
		) = unbounded();

		Self {
			_p: PhantomData,
			pipeline,
			pipeline_inputs,
			context,
			node_instances,
			node_status,
			edge_values,
			pool,
			send_data,
			receive_data,
			send_status,
			receive_status,
		}
	}

	/// Update this runner: process data and state changes that occured
	/// since we last called `run()`, and start any nodes that can now be started.
	///
	/// This method should be fairly fast, since it holds up the main thread.
	pub fn run(&mut self) -> Result<SingleRunnerState, PipelineError> {
		// Run nodes in a better order, and maybe skip a few.

		// Handle all changes that occured since we last called `run()`
		self.handle_all_messages()?;

		// Check every node. Initialize nodes that need to be initialized,
		// run nodes that need to be run. Nodes might be initialized and
		// run in the same cycle.
		let mut all_nodes_done = true;
		for (node, (_, _)) in self.pipeline.clone().graph.iter_nodes_idx() {
			if !self.node_status[node.as_usize()].is_done() {
				all_nodes_done = false;
			}

			self.try_start_node(node)?;
		}

		if all_nodes_done {
			return Ok(SingleRunnerState::Done);
		}

		return Ok(SingleRunnerState::Running);
	}

	/// Helper function, written here only for convenience.
	/// If we can add the node with index `n` to the thread pool, do so.
	fn try_start_node(&mut self, node: GraphNodeIdx) -> Result<(), PipelineError> {
		// Skip nodes we've already run and nodes that are running right now.
		let n = self.node_status.get(node.as_usize()).unwrap();
		if n.is_running() || n.is_done() {
			return Ok(());
		}

		// Skip nodes we can't run
		if self
			.pipeline
			.graph
			.edges_ending_at(node)
			.iter()
			.any(|edge_idx| {
				match self.edge_values.get(edge_idx.as_usize()).unwrap() {
					// Any input edges uninitialized => This node hasn't been run yet, and is waiting on another.
					EdgeValue::Uninitialized => true,
					// All edges have data => good to go!
					EdgeValue::Data(_) => false,
					// All `after` edges are ready => good to go!
					EdgeValue::AfterReady => false,
					// No edges should be consumed unless this node has been started
					EdgeValue::Consumed => {
						if !n.is_pending() {
							let n = self.pipeline.graph.get_node(node);
							unreachable!("Node {} tried to use consumed edge", n.0)
						} else {
							false
						}
					}
				}
			}) {
			return Ok(());
		}

		let mut prepare_inputs = || {
			if node == self.pipeline.input_node_idx {
				self.pipeline_inputs.clone()
			} else {
				// Initialize all with None, in case some are disconnected.
				let node_type = &self.pipeline.graph.get_node(node).1;
				let mut inputs = Vec::with_capacity(node_type.n_inputs(&self.context));
				for i in 0..node_type.n_inputs(&self.context) {
					let t = node_type.input_default_type(&self.context, i);
					inputs.push(PipelineData::new_empty(t));
				}

				// Now, fill input values
				for edge_idx in self.pipeline.graph.edges_ending_at(node) {
					let edge = &self.pipeline.graph.get_edge(*edge_idx).2;

					// Skip non-value-carrying edges
					if !edge.is_ptp() {
						continue;
					}

					let val = self.edge_values.get_mut(edge_idx.as_usize()).unwrap();
					match val {
						EdgeValue::Data(_) => {
							let x = std::mem::replace(val, EdgeValue::Consumed);
							*inputs.get_mut(edge.target_port().unwrap()).unwrap() = x.unwrap();
						}
						_ => unreachable!(),
					};
				}
				inputs
			}
		};

		let (n, node_instance) = &self.node_instances.get(node.as_usize()).unwrap();
		let node_instance = node_instance.clone();
		let n = n.clone();

		// Initialize this node if we need to
		if self
			.node_status
			.get_mut(node.as_usize())
			.unwrap()
			.is_notstarted()
		{
			println!("Init {}", n);
			let mut node_instance_locked = node_instance.lock().unwrap();
			*self.node_status.get_mut(node.as_usize()).unwrap() = NodeRunState::Running;
			let res = node_instance_locked.init(&self.context, prepare_inputs(), |port, data| {
				// This should never fail, since we never close the receiver.
				self.send_data.send((node, port, data)).unwrap();
				Ok(())
			});
			let done = res
				.as_ref()
				.ok()
				.map(|x| *x == PipelineNodeState::Done)
				.unwrap_or(true);
			self.send_status.send((node, res)).unwrap();

			// We don't need to run nodes that finished early
			if done {
				return Ok(());
			}

			// Process data and apply state changes
			// that this node's `init()`` produced.
			//
			// This MUST be done before running the node.
			self.handle_all_messages()?;
		}

		*self.node_status.get_mut(node.as_usize()).unwrap() = NodeRunState::Running;

		let ctx = self.context.clone();
		let send_data = self.send_data.clone();
		let send_status = self.send_status.clone();

		self.pool.execute(move || {
			println!("Run  {}", n);
			let mut node_instance = node_instance.lock().unwrap();
			let res = node_instance.run(&*ctx, |port, data| {
				// This should never fail, since we never close the receiver.
				send_data.send((node, port, data)).unwrap();
				Ok(())
			});
			send_status.send((node, res)).unwrap();
			println!("Done {}", n);
		});

		return Ok(());
	}

	/// Handle all messages nodes have sent up to this point.
	/// This MUST be done between successive calls of
	/// `run()` or `init()` on any one node.
	fn handle_all_messages(&mut self) -> Result<(), PipelineError> {
		for (node, port, data) in self.receive_data.try_iter() {
			// Fill every edge that is connected to this output port of this node
			for edge_idx in self
				.pipeline
				.graph
				.edges_starting_at(node)
				.iter()
				.filter(|edge_idx| {
					let edge = &self.pipeline.graph.get_edge(**edge_idx).2;
					edge.source_port() == Some(port)
				}) {
				*self.edge_values.get_mut(edge_idx.as_usize()).unwrap() =
					EdgeValue::Data(data.clone());
			}
		}

		for (node, res) in self.receive_status.try_iter() {
			match res {
				Err(x) => {
					return Err(x);
				}
				Ok(status) => {
					*self.node_status.get_mut(node.as_usize()).unwrap() =
						NodeRunState::NotRunning(status);

					if status.is_done() {
						// When a node finishes successfully, mark all
						// `after` edges that start at that node as "ready".
						for edge_idx in
							self.pipeline
								.graph
								.edges_starting_at(node)
								.iter()
								.filter(|edge_idx| {
									let edge = &self.pipeline.graph.get_edge(**edge_idx).2;
									edge.is_after()
								}) {
							*self.edge_values.get_mut(edge_idx.as_usize()).unwrap() =
								EdgeValue::AfterReady;
						}
					}
				}
			}
		}

		return Ok(());
	}
}
