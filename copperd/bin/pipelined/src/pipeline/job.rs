use copper_util::graph::util::GraphNodeIdx;
use crossbeam::channel::{unbounded, Receiver, Sender};
use pipelined_node_base::base::{
	Node, NodeDispatcher, NodeState, PipelineData, PipelineJobContext, PipelineNodeID,
	PipelinePortID, RunNodeError,
};
use smartstring::{LazyCompact, SmartString};
use std::{
	collections::{BTreeMap, VecDeque},
	error::Error,
	fmt::Display,
	sync::{Arc, Mutex},
	thread::JoinHandle,
	time::Instant,
};
use tracing::trace;

use crate::pipeline::spec::{EdgeSpec, PipelineSpec};

//
// MARK: Helpers
//

#[derive(Debug)]
#[allow(clippy::manual_non_exhaustive)]
pub struct PipelineJobError {
	// Prevent creation of this error outside this module
	_private: (),

	pub node: PipelineNodeID,
	pub error: RunNodeError,
}

impl Display for PipelineJobError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "Error in node `{}`: `{}`", self.node, self.error)
	}
}

impl Error for PipelineJobError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		Some(&self.error)
	}
}

/// The state of a [`PipelineJob`]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PipelineJobState {
	/// Nodes are running, not done yet
	Running,

	/// Pipeline is done, this runner may be dropped.
	Done,
}

struct NodeInstanceContainer<DataType: PipelineData> {
	/// The node's id
	id: PipelineNodeID,

	/// A queue of inputs to send to this node
	// This will be `None` only if the node is done,
	/// since done nodes don't take input.
	input_queue: Option<VecDeque<(PipelinePortID, DataType)>>,

	/// This node's status
	state: NodeRunState,

	/// When we last ran this node
	last_run: Instant,

	/// The node. This will be `None` if the node is done,
	/// so that its resources are dropped.
	node: Arc<Mutex<Option<Box<dyn Node<DataType>>>>>,
}

/// A wrapper around [`PipelineNodeState`] that keeps
/// track if a certain node is running *right now*.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum NodeRunState {
	/// This node is currently queued in the thread pool.
	Running,

	/// This node is not actively running
	NotRunning(NodeState),
}

impl NodeRunState {
	/// Is this node [`NodeRunState::Running`]?
	pub fn is_running(&self) -> bool {
		matches!(self, Self::Running)
	}

	/// Is this node `NodeRunState::NotRunning(PipelineNodestate::Done)`?
	pub fn is_done(&self) -> bool {
		matches!(self, Self::NotRunning(NodeState::Done))
	}
}

#[derive(Debug)]
pub(super) enum EdgeState {
	/// This is a normal data edge
	Data,

	// This is an Edge::After that is waiting for it's source node
	AfterWaiting,

	// This is an Edge::After whose source node has finished running.
	AfterReady,
}

//
// MARK: PipelineJob
//

pub struct PipelineJob<DataType: PipelineData, ContextType: PipelineJobContext<DataType>> {
	/// The pipeline we're running
	pipeline: Arc<PipelineSpec<DataType, ContextType>>,

	pub(crate) context: ContextType,

	/// Mutable instances of each node in this pipeline
	node_instances: Vec<NodeInstanceContainer<DataType>>,

	/// The value each edge in this pipeline carries
	edge_values: Vec<EdgeState>,

	/// A threadpool of node runners
	workers: Vec<Option<JoinHandle<()>>>,

	/// A copy of this is given to every node.
	/// Nodes send outputs here.
	send_data: Sender<(
		// The node that sent this message
		GraphNodeIdx,
		// The port index of this output
		PipelinePortID,
		// The data that output produced
		DataType,
	)>,

	/// A receiver for node output data
	receive_data: Receiver<(
		// The node that sent this message
		GraphNodeIdx,
		// The port index of this output
		PipelinePortID,
		// The data that output produced
		DataType,
	)>,

	/// A message is sent here whenever a node finishes running.
	send_status: Sender<(
		// The node that sent this status
		GraphNodeIdx,
		// The status that was sent
		Result<NodeState, RunNodeError>,
	)>,

	/// A receiver for node status messages
	receive_status: Receiver<(
		// The node that sent this status
		GraphNodeIdx,
		// The status that was sent
		Result<NodeState, RunNodeError>,
	)>,

	node_run_offset: usize,
}

impl<DataType: PipelineData, ContextType: PipelineJobContext<DataType>> Drop
	for PipelineJob<DataType, ContextType>
{
	fn drop(&mut self) {
		for i in &mut self.workers {
			if let Some(t) = i.take() {
				t.join().unwrap();
			};
		}
	}
}

impl<DataType: PipelineData, ContextType: PipelineJobContext<DataType>>
	PipelineJob<DataType, ContextType>
{
	/// Get the pipeline this job is running
	pub fn get_pipeline(&self) -> &PipelineSpec<DataType, ContextType> {
		&self.pipeline
	}

	/// Get the current state of all nodes in this job
	/// Returns `None` if an unknown node name is provided.
	pub fn get_node_status(&self, node: &PipelineNodeID) -> Option<(bool, NodeState)> {
		self.node_instances
			.iter()
			.find(|x| &x.id == node)
			.map(|x| match &x.state {
				NodeRunState::Running => (true, NodeState::Pending("is running")),
				NodeRunState::NotRunning(x) => (false, *x),
			})
	}

	pub fn get_input(&self) -> &BTreeMap<SmartString<LazyCompact>, DataType> {
		self.context.get_input()
	}

	pub(super) fn new(
		pipeline: Arc<PipelineSpec<DataType, ContextType>>,
		dispatcher: &NodeDispatcher<DataType, ContextType>,
		context: ContextType,
		worker_threads: usize,
	) -> Self {
		let instant_now = Instant::now();
		trace!(message = "Making node instances", pipeline_name = ?pipeline.name);
		let node_instances = pipeline
			.graph
			.iter_nodes_idx()
			.map(|(idx, node_data)| {
				let node = dispatcher
					.init_node(
						&context,
						&node_data.node_type,
						&node_data.node_params,
						node_data.id.id(),
					)
					// We already checked node type in `Builder`
					.unwrap()
					.unwrap();

				let mut input_queue = VecDeque::new();

				// Send empty data to disconnected inputs
				let mut port_is_empty = node
					.get_info()
					.inputs()
					.iter()
					.map(|(id, _)| (id.clone(), true))
					.collect::<BTreeMap<_, _>>();
				for i in pipeline.graph.edges_ending_at(idx) {
					let edge = &pipeline.graph.get_edge(*i).2;
					if edge.is_after() {
						continue;
					}
					*port_is_empty.get_mut(&edge.target_port().unwrap()).unwrap() = false;
				}
				for (port, is_empty) in port_is_empty.into_iter() {
					if is_empty {
						input_queue.push_back((
							port.clone(),
							DataType::disconnected(*node.get_info().inputs().get(&port).unwrap()),
						));
					}
				}

				NodeInstanceContainer {
					id: node_data.id.clone(),
					input_queue: Some(input_queue),
					last_run: instant_now,
					state: NodeRunState::NotRunning(NodeState::Pending("not started")),
					node: Arc::new(Mutex::new(Some(node))),
				}
			})
			.collect::<Vec<_>>();

		// The data inside each edge.
		// We consume node data once it is read so that unneeded memory may be freed.
		trace!(message = "Initializing edges", pipeline_name = ?pipeline.name);
		let edge_values = {
			pipeline
				.graph
				.iter_edges()
				.map(|(_, _, x)| match x {
					EdgeSpec::After => EdgeState::AfterWaiting,
					EdgeSpec::PortToPort(_) => EdgeState::Data,
				})
				.collect::<Vec<_>>()
		};

		// Channel for node data. Nodes send their outputs here once they are ready.
		//
		// Contents are (node index, port index, data)
		#[allow(clippy::type_complexity)]
		let (send_data, receive_data): (
			Sender<(GraphNodeIdx, PipelinePortID, DataType)>,
			Receiver<(GraphNodeIdx, PipelinePortID, DataType)>,
		) = unbounded();

		// Channel for node status. A node's return status is sent here when it finishes.
		//
		// Contents are (node index, result of `node.run()`)
		#[allow(clippy::type_complexity)]
		let (send_status, receive_status): (
			Sender<(GraphNodeIdx, Result<NodeState, RunNodeError>)>,
			Receiver<(GraphNodeIdx, Result<NodeState, RunNodeError>)>,
		) = unbounded();

		Self {
			context,
			pipeline,
			node_instances,
			edge_values,
			workers: (0..worker_threads).map(|_| None).collect(),
			send_data,
			receive_data,
			send_status,
			receive_status,
			node_run_offset: 0,
		}
	}

	/// Update this job: process state changes that occurred since we last called `run()`,
	/// deliver new data, and start nodes that should be started.
	///
	/// This method should be called often, but not too often.
	/// All computation is done in a thread pool, `run()`'s responsibility
	/// is to update state and schedule new nodes.
	pub(super) fn run(&mut self) -> Result<PipelineJobState, PipelineJobError> {
		// Run nodes in a better order, and maybe skip a few.

		// Handle all changes that occurred since we last called `run()`
		self.handle_all_messages()?;

		// Clean up threads that finished since we last called `run()`
		for w in &mut self.workers {
			if w.is_some() && w.as_ref().unwrap().is_finished() {
				w.take().unwrap().join().unwrap();
			}
		}

		// Check every node. Initialize nodes that need to be initialized,
		// run nodes that need to be run. Nodes might be initialized and
		// run in the same cycle.
		let mut all_nodes_done = true;
		for i in 0..self.pipeline.graph.len_nodes() {
			let i = (i + self.node_run_offset) % self.pipeline.graph.len_nodes();
			if !self.node_instances[i].state.is_done() {
				all_nodes_done = false;
			}

			self.node_instances[i].last_run = Instant::now();
			self.try_start_node(GraphNodeIdx::from_usize(i))?;
		}
		self.node_run_offset += 1;
		self.node_run_offset %= self.node_instances.len();

		if all_nodes_done {
			return Ok(PipelineJobState::Done);
		}

		return Ok(PipelineJobState::Running);
	}

	/// Helper function, written here only for convenience.
	/// If we can add the node with index `n` to the thread pool, do so.
	fn try_start_node(&mut self, node: GraphNodeIdx) -> Result<(), PipelineJobError> {
		// Skip nodes we've already run and nodes that are running right now.
		let n = self.node_instances[node.as_usize()].state;
		let node_id = self.node_instances[node.as_usize()].id.clone();
		if n.is_running() || n.is_done() {
			return Ok(());
		}

		// Do nothing if there are no free workers
		if self.workers.iter().all(|x| x.is_some()) {
			return Ok(());
		}

		// Send all pending input to node
		{
			let node_instance_container = &mut self.node_instances[node.as_usize()];
			let node_instance = node_instance_container.node.clone();
			let mut locked_node = node_instance.lock().unwrap();

			while !node_instance_container
				.input_queue
				.as_ref()
				.unwrap()
				.is_empty()
			{
				let data = node_instance_container
					.input_queue
					.as_mut()
					.unwrap()
					.pop_front()
					.unwrap();

				locked_node
					.as_mut()
					.unwrap()
					.take_input(data.0, data.1)
					.map_err(|error| PipelineJobError {
						_private: (),
						node: node_id.clone(),
						error,
					})?;
			}
		}

		// Nodes that are blocked by an "after" edge receive input, but are not started.
		if self
			.pipeline
			.graph
			.edges_ending_at(node)
			.iter()
			.any(
				|edge_idx| match self.edge_values.get(edge_idx.as_usize()).unwrap() {
					EdgeState::Data => false,
					EdgeState::AfterWaiting => true,
					EdgeState::AfterReady => false,
				},
			) {
			return Ok(());
		}

		self.node_instances[node.as_usize()].state = NodeRunState::Running;
		let node_instance_container = &mut self.node_instances[node.as_usize()];
		let node_instance = node_instance_container.node.clone();
		let node_id = node_instance_container.id.clone();
		let send_data = self.send_data.clone();
		let send_status = self.send_status.clone();
		let pipeline_name = self.pipeline.name.clone();

		if node_instance
			.try_lock()
			.unwrap()
			.as_ref()
			.unwrap()
			.quick_run()
		{
			trace!(
				message = "Quick-running node",
				pipeline_name = ?pipeline_name,
				node_id = node_id.to_string()
			);

			// Panics if mutex is locked. This is intentional, only one thread should have this at a time.
			// We use a mutex only for interior mutability.
			let mut node_instance_opt = node_instance.try_lock().unwrap();
			let node_instance = node_instance_opt.as_mut().unwrap();

			let res = node_instance.run(&|port, data| {
				// This should never fail, since we never close the receiver.
				send_data.send((node, port, data)).unwrap();
				Ok(())
			});

			trace!(
				message = "Node finished",
				node = node_id.to_string(),
				pipeline_name = ?pipeline_name,
				status=?res.as_ref().unwrap()
			);
			send_status.send((node, res)).unwrap();
		} else {
			let mut worker = Some(std::thread::spawn(move || {
				trace!(
					message = "Running node",
					pipeline_name = ?pipeline_name,
					node = node_id.to_string()
				);

				// Panics if mutex is locked. This is intentional, only one thread should have this at a time.
				// We use a mutex only for interior mutability.
				let mut node_instance_opt = node_instance.try_lock().unwrap();
				let node_instance = node_instance_opt.as_mut().unwrap();

				let res = node_instance.run(&|port, data| {
					// This should never fail, since we never close the receiver.
					send_data.send((node, port, data)).unwrap();
					Ok(())
				});

				trace!(
					message = "Node finished",
					node = node_id.to_string(),
					pipeline_name = ?pipeline_name,
					status=?res.as_ref()
				);
				send_status.send((node, res)).unwrap();
			}));

			for w in &mut self.workers {
				if w.is_none() {
					*w = worker.take();
					break;
				}
			}

			assert!(worker.is_none());
		}

		return Ok(());
	}

	/// Handle all messages nodes have sent up to this point.
	/// This MUST be done between successive calls of
	/// `run()` on any one node.
	fn handle_all_messages(&mut self) -> Result<(), PipelineJobError> {
		for (node, port, data) in self.receive_data.try_iter() {
			// Send data to all inputs connected to this output
			for edge_idx in self.pipeline.graph.edges_starting_at(node) {
				let (_, to_node, edge) = &self.pipeline.graph.get_edge(*edge_idx);
				if !(edge.is_ptp() && edge.source_port().as_ref() == Some(&port)) {
					continue;
				}
				let node = self.node_instances.get_mut(to_node.as_usize()).unwrap();

				// Don't give input to nodes that are done
				if !node.state.is_done() {
					node.input_queue
						.as_mut()
						.unwrap()
						.push_back((edge.target_port().unwrap(), data.clone()));
				}
			}
		}

		for (node, res) in self.receive_status.try_iter() {
			match res {
				Err(x) => {
					return Err(PipelineJobError {
						_private: (),
						node: self.node_instances[node.as_usize()].id.clone(),
						error: x,
					});
				}
				Ok(status) => {
					self.node_instances[node.as_usize()].state = NodeRunState::NotRunning(status);

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
						trace!(
							message = "Dropped node",
							pipeline_name = ?self.pipeline.name,
							node = self.node_instances[node.as_usize()].id.to_string(),
						);

						let mut x = self.node_instances[node.as_usize()].node.lock().unwrap();
						drop(x.take());
						drop(x);
						drop(self.node_instances[node.as_usize()].input_queue.take());

						// Quick sanity check
						assert!(self.node_instances[node.as_usize()]
							.node
							.try_lock()
							.unwrap()
							.is_none());
						assert!(self.node_instances[node.as_usize()].input_queue.is_none());
					}
				}
			}
		}

		return Ok(());
	}
}
