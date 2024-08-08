use crossbeam::channel::{unbounded, Receiver, Sender};
use std::{
	collections::VecDeque,
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
pub(super) enum SingleJobState {
	/// Nodes are running, not done yet
	Running,

	/// Pipeline is done, this runner may be dropped.
	Done,
}

/// An instance of a single running pipeline
pub struct PipelineSingleJob<StubType: PipelineNodeStub> {
	_p: PhantomData<StubType>,

	/// The pipeline we're running
	pipeline: Arc<Pipeline<StubType>>,

	/// The input we ran this pipeline with
	input: Vec<SDataType<StubType>>,

	/// Mutable instances of each node in this pipeline
	node_instances: Vec<(
		// The node's label
		PipelineNodeLabel,
		// A queue of inputs to send to this node
		VecDeque<(usize, SDataType<StubType>)>,
		// This node's status
		NodeRunState,
		// The node
		Arc<Mutex<Option<SNodeType<StubType>>>>,
	)>,

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

impl<StubType: PipelineNodeStub> Drop for PipelineSingleJob<StubType> {
	fn drop(&mut self) {
		self.pool.join();
	}
}

impl<StubType: PipelineNodeStub> PipelineSingleJob<StubType> {
	/// Get the pipeline this job is running
	pub fn get_pipeline(&self) -> &Pipeline<StubType> {
		&*self.pipeline
	}

	/// Get the current state of all nodes in this job
	/// Returns `None` if an unknown node name is provided.
	pub fn get_node_status(&self, node: &PipelineNodeLabel) -> Option<(bool, PipelineNodeState)> {
		self.node_instances
			.iter()
			.find(|(label, _, _, _)| label == node)
			.map(|(_, _, status, _)| match status {
				NodeRunState::Running => (true, PipelineNodeState::Pending("is running")),
				NodeRunState::NotRunning(x) => (false, *x),
			})
	}

	pub fn get_input(&self) -> &Vec<SDataType<StubType>> {
		&self.input
	}
}

impl<'a, StubType: PipelineNodeStub> PipelineSingleJob<StubType> {
	/// Make a new [`PipelineSingleRunner`]
	pub(super) fn new(
		config: &'a PipelineRunConfig,
		context: Arc<<StubType::NodeType as PipelineNode>::NodeContext>,
		pipeline: Arc<Pipeline<StubType>>,
		input: Vec<SDataType<StubType>>,
	) -> Self {
		assert!(
			input.len()
				== pipeline
					.graph
					.get_node(pipeline.input_node_idx)
					.1
					.n_inputs(&context)
		);

		debug!(source = "single", summary = "Making node instances");
		let node_instances = pipeline
			.graph
			.iter_nodes_idx()
			.map(|(idx, (name, x))| {
				let mut input_queue = VecDeque::new();
				// Pass pipeline inputs to input node immediately
				if idx == pipeline.input_node_idx {
					for (i, d) in input.iter().enumerate() {
						input_queue.push_back((i, d.clone()));
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
							input_queue.push_back((i, SDataType::<StubType>::new_empty(t)));
						}
					}
				}

				(
					name.clone(),
					input_queue,
					NodeRunState::NotRunning(PipelineNodeState::Pending("not started")),
					Arc::new(Mutex::new(Some(x.build(&context, name.into())))),
				)
			})
			.collect::<Vec<_>>();

		// The data inside each edge.
		// We consume node data once it is read so that unneeded memory may be freed.
		debug!(source = "single", summary = "Initializing edges");
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
			input,
			node_instances,
			edge_values,
			pool,
			send_data,
			receive_data,
			send_status,
			receive_status,
		}
	}
}

impl<'a, StubType: PipelineNodeStub> PipelineSingleJob<StubType> {
	/// Update this job: process state changes that occured since we last called `run()`,
	/// deliver new data, and start nodes that should be started.
	///
	/// This method should be called often, but not too often.
	/// All computation is done in a thread pool, `run()`'s responsibility
	/// is to update state and schedule new nodes.
	pub(super) fn run(&mut self) -> Result<SingleJobState, SErrorType<StubType>> {
		// Run nodes in a better order, and maybe skip a few.

		// Handle all changes that occured since we last called `run()`
		self.handle_all_messages()?;

		// Check every node. Initialize nodes that need to be initialized,
		// run nodes that need to be run. Nodes might be initialized and
		// run in the same cycle.
		let mut all_nodes_done = true;
		for (node, (_, _)) in self.pipeline.clone().graph.iter_nodes_idx() {
			if !self.node_instances[node.as_usize()].2.is_done() {
				all_nodes_done = false;
			}

			self.try_start_node(node)?;
		}

		if all_nodes_done {
			return Ok(SingleJobState::Done);
		}

		return Ok(SingleJobState::Running);
	}

	/// Helper function, written here only for convenience.
	/// If we can add the node with index `n` to the thread pool, do so.
	fn try_start_node(&mut self, node: GraphNodeIdx) -> Result<(), SErrorType<StubType>> {
		// Skip nodes we've already run and nodes that are running right now.
		let n = self.node_instances[node.as_usize()].2;
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
					// We don't care about theseend_input,
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

		self.node_instances[node.as_usize()].2 = NodeRunState::Running;
		let (node_label, input, _, node_instance) = &mut self.node_instances[node.as_usize()];
		let node_instance = node_instance.clone();
		let node_label = node_label.clone();

		let send_data = self.send_data.clone();
		let send_status = self.send_status.clone();

		let mut locked_node = node_instance.lock().unwrap();

		// Send new input to node
		while !input.is_empty() {
			let data = input.pop_front().unwrap();
			locked_node
				.as_mut()
				.unwrap()
				.take_input(data, |port, data| {
					// This should never fail, since we never close the receiver.
					send_data.send((node, port, data)).unwrap();
					Ok(())
				})?;
		}
		self.handle_all_messages()?;
		drop(locked_node);

		if node_instance
			.try_lock()
			.unwrap()
			.as_ref()
			.unwrap()
			.quick_run()
		{
			debug!(
				source = "pipeline",
				summary = "Quick-running node",
				node = node_label.to_string()
			);

			// Panics if mutex is locked. This is intentional, only one thread should have this at a time.
			// We use a mutex only for interior mutability.
			let mut node_instance_opt = node_instance.try_lock().unwrap();
			let node_instance = node_instance_opt.as_mut().unwrap();

			let res = node_instance.run(|port, data| {
				// This should never fail, since we never close the receiver.
				send_data.send((node, port, data)).unwrap();
				Ok(())
			});

			debug!(
				source = "pipeline",
				summary = "Node finished",
				node = node_label.to_string(),
				status=?res.as_ref().unwrap()
			);
			send_status.send((node, res)).unwrap();
		} else {
			self.pool.execute(move || {
				debug!(
					source = "pipeline",
					summary = "Running node",
					node = node_label.to_string()
				);

				// Panics if mutex is locked. This is intentional, only one thread should have this at a time.
				// We use a mutex only for interior mutability.
				let mut node_instance_opt = node_instance.try_lock().unwrap();
				let node_instance = node_instance_opt.as_mut().unwrap();

				let res = node_instance.run(|port, data| {
					// This should never fail, since we never close the receiver.
					send_data.send((node, port, data)).unwrap();
					Ok(())
				});

				debug!(
					source = "pipeline",
					summary = "Node finished",
					node = node_label.to_string(),
					status=?res.as_ref().unwrap()
				);
				send_status.send((node, res)).unwrap();
			});
		}

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
					.get_mut(to_node.as_usize())
					.unwrap()
					.1
					.push_back((edge.target_port().unwrap(), data.clone()));
			}
		}

		for (node, res) in self.receive_status.try_iter() {
			match res {
				Err(x) => {
					return Err(x);
				}
				Ok(status) => {
					self.node_instances[node.as_usize()].2 = NodeRunState::NotRunning(status);

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
						debug!(
							source = "pipeline",
							summary = "Dropped node",
							node = self.node_instances[node.as_usize()].0.to_string(),
						);

						let mut x = self.node_instances[node.as_usize()].3.lock().unwrap();
						drop(x.take());

						// Quick sanity check
						drop(x);
						assert!(self.node_instances[node.as_usize()]
							.3
							.try_lock()
							.unwrap()
							.is_none())
					}
				}
			}
		}

		return Ok(());
	}
}
