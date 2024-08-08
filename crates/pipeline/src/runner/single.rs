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
	pipeline::Pipeline,
	syntax::labels::PipelineNodeLabel,
};

/// An instance of a single running pipeline
pub(super) struct PipelineSingleRunner<StubType: PipelineNodeStub> {
	_p: PhantomData<StubType>,

	/// The pipeline we're running
	pipeline: Arc<Pipeline<StubType>>,

	/// The context for this pipeline
	context: Arc<<StubType::NodeType as PipelineNode>::NodeContext>,

	/// Mutable instances of each node in this pipeline
	node_instances: Vec<(
		PipelineNodeLabel,
		Arc<Mutex<<StubType as PipelineNodeStub>::NodeType>>,
	)>,

	/// Each node's status
	/// (indices match `node_instances`)
	node_status: Vec<NodeRunState>,

	/// The value each edge in this pipeline carries
	edge_values:
		Vec<EdgeValue<<<StubType as PipelineNodeStub>::NodeType as PipelineNode>::DataType>>,

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
		<<StubType as PipelineNodeStub>::NodeType as PipelineNode>::DataType,
	)>,

	/// A receiver for node output data
	receive_data: Receiver<(
		// The node that sent this message
		GraphNodeIdx,
		// The port index of this output
		usize,
		// The data that output produced
		<<StubType as PipelineNodeStub>::NodeType as PipelineNode>::DataType,
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
		context: Arc<<StubType::NodeType as PipelineNode>::NodeContext>,
		pipeline: Arc<Pipeline<StubType>>,
		node_runners: usize,
	) -> Self {
		let node_instances = pipeline
			.graph
			.iter_nodes()
			.map(|(name, x)| {
				(
					name.clone(),
					Arc::new(Mutex::new(x.build(context.clone(), name.into()))),
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
			Sender<(
				GraphNodeIdx,
				usize,
				<StubType::NodeType as PipelineNode>::DataType,
			)>,
			Receiver<(
				GraphNodeIdx,
				usize,
				<StubType::NodeType as PipelineNode>::DataType,
			)>,
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

	pub fn run(
		&mut self,
		pipeline_inputs: Vec<<StubType::NodeType as PipelineNode>::DataType>,
	) -> Result<(), PipelineError> {
		assert!(
			pipeline_inputs.len()
				== self
					.pipeline
					.graph
					.get_node(self.pipeline.input_node_idx)
					.1
					.inputs(self.context.clone())
					.len()
		);

		// Check every node.
		// TODO: write a smarter scheduler.
		loop {
			let mut finished_all_outputs = true;
			for (node, (_, _)) in self.pipeline.clone().graph.iter_nodes_idx() {
				if !self.node_status[node.as_usize()].is_done() {
					finished_all_outputs = false;
				}

				self.try_run_node(&pipeline_inputs, node)?;
			}

			self.handle_all_messages()?;

			// TODO: end condition.
			// TODO: after moves to END of pipeline node
			// TODO: handle all messages?
			// TODO: clean up threads?
			// TODO: quick node run, no thread

			if finished_all_outputs {
				return Ok(());
			}
		}
	}

	/// Helper function, written here only for convenience.
	/// Try to run the node with index `n`.
	fn try_run_node(
		&mut self,
		pipeline_inputs: &Vec<<StubType::NodeType as PipelineNode>::DataType>,
		node: GraphNodeIdx,
	) -> Result<(), PipelineError> {
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
				pipeline_inputs.clone()
			} else {
				// Initialize all with None, in case some are disconnected.
				let node_type = &self.pipeline.graph.get_node(node).1;
				let mut inputs = Vec::with_capacity(node_type.inputs(self.context.clone()).len());
				for (_, t) in node_type.inputs(self.context.clone()).iter() {
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

		// We MUST handle all status codes before re-running a node.
		// TODO: clean up scheduler

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
			let res =
				node_instance_locked.init(self.context.clone(), prepare_inputs(), |port, data| {
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
		} else {
			*self.node_status.get_mut(node.as_usize()).unwrap() = NodeRunState::Running;

			let ctx = self.context.clone();
			let send_data = self.send_data.clone();
			let send_status = self.send_status.clone();

			self.pool.execute(move || {
				println!("Run  {}", n);
				let mut node_instance = node_instance.lock().unwrap();
				let res = node_instance.run(ctx, |port, data| {
					// This should never fail, since we never close the receiver.
					send_data.send((node, port, data)).unwrap();
					Ok(())
				});
				send_status.send((node, res)).unwrap();
				println!("Done {}", n);
			});
		}

		return Ok(());
	}

	/// Handle all messages nodes have sent up to this point.
	fn handle_all_messages(&mut self) -> Result<(), PipelineError> {
		for (node, port, data) in self.receive_data.try_iter() {
			// Fill every edge that is connected to
			// this output port of this node
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
						// `after` edges that start at it as "ready".
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
