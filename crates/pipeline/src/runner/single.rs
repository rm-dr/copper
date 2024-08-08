use crossbeam::channel::{unbounded, Receiver, Sender};
use std::{
	marker::PhantomData,
	sync::{Arc, Mutex},
};
use threadpool::ThreadPool;
use tracing::debug;

use super::{
	runner::PipelineRunConfig,
	util::{EdgeState, NodeRunState},
};
use crate::{
	api::{PipelineData, PipelineNode, PipelineNodeState, PipelineNodeStub},
	graph::util::GraphNodeIdx,
	labels::PipelineNodeLabel,
	pipeline::{Pipeline, PipelineEdge},
	SDataType, SErrorType, SNodeType,
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

	/// Mutable instances of each node in this pipeline
	node_instances: Vec<(
		// The node's label
		PipelineNodeLabel,
		// Where to send this node's inputs
		Sender<(usize, SDataType<StubType>)>,
		// The node
		Arc<Mutex<Option<SNodeType<StubType>>>>,
	)>,

	/// Each node's status
	/// (indices match `node_instances`)
	node_status: Vec<NodeRunState>,

	/// The value each edge in this pipeline carries
	edge_values: Vec<EdgeState>,

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
		Result<PipelineNodeState, SErrorType<StubType>>,
	)>,

	/// A receiver for node status messages
	receive_status: Receiver<(
		// The node that sent this status
		GraphNodeIdx,
		// The status that was sent
		Result<PipelineNodeState, SErrorType<StubType>>,
	)>,
}

impl<'a, StubType: PipelineNodeStub> PipelineSingleRunner<StubType> {
	/// Make a new [`PipelineSingleRunner`]
	pub fn new(
		config: &'a PipelineRunConfig,
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
			.iter_nodes_idx()
			.map(|(idx, (name, x))| {
				#[allow(clippy::type_complexity)]
				let (send_input, receive_input): (
					Sender<(usize, SDataType<StubType>)>,
					Receiver<(usize, SDataType<StubType>)>,
				) = unbounded();

				// Pass pipeline inputs to input node immediately
				if idx == pipeline.input_node_idx {
					for (i, d) in pipeline_inputs.iter().enumerate() {
						send_input.send((i, d.clone())).unwrap();
					}
				} else {
					// Send empty data to disconnected inputs
					let mut port_is_empty =
						(0..x.n_inputs(&*context)).map(|_| true).collect::<Vec<_>>();
					for i in pipeline.graph.edges_ending_at(idx) {
						let edge = &pipeline.graph.get_edge(*i).2;
						if edge.is_after() {
							continue;
						}
						port_is_empty[edge.target_port().unwrap()] = false;
					}
					for (i, e) in port_is_empty.into_iter().enumerate() {
						if e {
							let t = x.input_default_type(&*context, i);
							send_input
								.send((i, SDataType::<StubType>::new_empty(t)))
								.unwrap();
						}
					}
				}

				(
					name.clone(),
					send_input,
					Arc::new(Mutex::new(Some(x.build(
						&context,
						name.into(),
						receive_input,
					)))),
				)
			})
			.collect::<Vec<_>>();

		// The data inside each edge.
		// We consume node data once it is read so that unneeded memory may be freed.
		let edge_values = {
			pipeline
				.graph
				.iter_edges()
				.map(|(_, _, x)| match x {
					PipelineEdge::After => EdgeState::AfterWaiting,
					PipelineEdge::PortToPort(_) => EdgeState::Data,
				})
				.collect::<Vec<_>>()
		};

		// Keep track of nodes we have already run.
		// We already initialized all input edges, so mark that node `true`.
		let node_status = pipeline
			.graph
			.iter_nodes_idx()
			.map(|_| NodeRunState::NotRunning(PipelineNodeState::Pending))
			.collect::<Vec<_>>();

		// Threadpool we'll use to run nodes
		let pool = threadpool::Builder::new()
			.num_threads(config.node_threads)
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
			Sender<(
				GraphNodeIdx,
				Result<PipelineNodeState, SErrorType<StubType>>,
			)>,
			Receiver<(
				GraphNodeIdx,
				Result<PipelineNodeState, SErrorType<StubType>>,
			)>,
		) = unbounded();

		Self {
			_p: PhantomData,
			pipeline,
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
	pub fn run(&mut self) -> Result<SingleRunnerState, SErrorType<StubType>> {
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
	fn try_start_node(&mut self, node: GraphNodeIdx) -> Result<(), SErrorType<StubType>> {
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
					// We don't care about these
					EdgeState::Data => false,
					// If any `after` edges are waiting, we can't start.
					// Be careful with these, they can cause a deadlock when
					// used with `Blob` data.
					EdgeState::AfterWaiting => true,
					// All `after` edges are ready => good to go!
					EdgeState::AfterReady => false,
				}
			}) {
			return Ok(());
		}

		let (n, _, node_instance) = &self.node_instances.get(node.as_usize()).unwrap();
		let node_instance = node_instance.clone();
		let n = n.clone();

		*self.node_status.get_mut(node.as_usize()).unwrap() = NodeRunState::Running;
		let ctx = self.context.clone();
		let send_data = self.send_data.clone();
		let send_status = self.send_status.clone();

		self.pool.execute(move || {
			debug!(
				source = "pipeline",
				summary = "Starting node",
				node = n.to_string()
			);

			// Panics if mutex is locked. This is intentional, only one thread should have this at a time.
			// We use a mutex only for interior mutability.
			let mut node_instance_opt = node_instance.lock().unwrap();
			let node_instance = node_instance_opt.as_mut().unwrap();
			let res = node_instance.take_input(|port, data| {
				// This should never fail, since we never close the receiver.
				send_data.send((node, port, data)).unwrap();
				Ok(())
			});

			if let Err(res) = res {
				debug!(
					source = "pipeline",
					summary = "Node finished with error",
					node = n.to_string(),
					error=?res
				);
				send_status.send((node, Err(res))).unwrap();
			} else {
				let res = node_instance.run(&*ctx, |port, data| {
					// This should never fail, since we never close the receiver.
					send_data.send((node, port, data)).unwrap();
					Ok(())
				});

				debug!(
					source = "pipeline",
					summary = "Node finished",
					node = n.to_string(),
					status=?res.as_ref().unwrap()
				);
				send_status.send((node, res)).unwrap();
			}
		});

		return Ok(());
	}

	/// Handle all messages nodes have sent up to this point.
	/// This MUST be done between successive calls of
	/// `run()` on any one node.
	fn handle_all_messages(&mut self) -> Result<(), SErrorType<StubType>> {
		for (node, port, data) in self.receive_data.try_iter() {
			// Send data to all inputs connected to this output
			for edge_idx in self.pipeline.graph.edges_starting_at(node) {
				let (_, to_node, edge) = &self.pipeline.graph.get_edge(*edge_idx);
				if !(edge.is_ptp() && edge.source_port() == Some(port)) {
					continue;
				}

				// Send data to target port
				self.node_instances
					.get(to_node.as_usize())
					.unwrap()
					.1
					.send((edge.target_port().unwrap(), data.clone()))
					.unwrap();
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
								EdgeState::AfterReady;
						}

						// Drop any node instance that is done.
						// This cleans up all resources that node used, and prevents
						// deadlocks caused by dangling Blob receivers.
						//
						// This intentionally panics if the mutex is already locked.
						// That should never happen!
						println!("drop {}", self.node_instances[node.as_usize()].0);
						let mut x = self.node_instances[node.as_usize()].2.try_lock().unwrap();
						drop(x.take());
					}
				}
			}
		}

		return Ok(());
	}
}
