use crossbeam::channel::{unbounded, Receiver, Sender};
use smartstring::{LazyCompact, SmartString};
use std::{
	fs::File,
	io::Read,
	marker::PhantomData,
	path::Path,
	sync::{Arc, Mutex},
};
use threadpool::ThreadPool;

use super::util::{EdgeValue, NodeRunState};
use crate::{
	errors::PipelineError,
	graph::util::GraphNodeIdx,
	node::{PipelineData, PipelineNode, PipelineNodeState, PipelineNodeStub},
	pipeline::Pipeline,
	syntax::{errors::PipelinePrepareError, labels::PipelineNodeLabel, spec::PipelineSpec},
};

/// A prepared data processing pipeline.
/// This is guaranteed to be correct:
/// no dependency cycles, no port type mismatch, etc
pub struct PipelineRunner<StubType: PipelineNodeStub> {
	_p: PhantomData<StubType>,
	pipelines: Vec<(SmartString<LazyCompact>, Arc<Pipeline<StubType>>)>,
	node_runners: usize,
}

impl<StubType: PipelineNodeStub> PipelineRunner<StubType> {
	pub fn new(node_runners: usize) -> Self {
		Self {
			_p: PhantomData,
			pipelines: Vec::new(),
			node_runners,
		}
	}

	pub fn add_pipeline(
		&mut self,
		ctx: Arc<<StubType::NodeType as PipelineNode>::NodeContext>,
		path: &Path,
		pipeline_name: String,
	) -> Result<(), PipelinePrepareError> {
		let mut f =
			File::open(path).map_err(|error| PipelinePrepareError::CouldNotOpenFile { error })?;

		let mut s: String = Default::default();

		f.read_to_string(&mut s)
			.map_err(|error| PipelinePrepareError::CouldNotReadFile { error })?;

		let spec: PipelineSpec<StubType> = toml::from_str(&s)
			.map_err(|error| PipelinePrepareError::CouldNotParseFile { error })?;

		let p = spec.prepare(ctx.clone(), pipeline_name.clone(), &self.pipelines)?;
		self.pipelines.push((pipeline_name.into(), Arc::new(p)));
		return Ok(());
	}

	pub fn get_pipeline(
		&self,
		pipeline_name: SmartString<LazyCompact>,
	) -> Option<Arc<Pipeline<StubType>>> {
		self.pipelines
			.iter()
			.find(|(x, _)| x == &pipeline_name)
			.map(|(_, x)| x.clone())
	}
}

impl<StubType: PipelineNodeStub> PipelineRunner<StubType> {
	/// Run a pipeline to completion.
	pub fn run(
		&mut self,
		ctx: Arc<<StubType::NodeType as PipelineNode>::NodeContext>,
		pipeline_name: SmartString<LazyCompact>,
		pipeline_inputs: Vec<<StubType::NodeType as PipelineNode>::DataType>,
	) -> Result<(), PipelineError> {
		let pipeline = self.get_pipeline(pipeline_name).unwrap();

		// TODO: async-like scheduler with node state
		let mut node_instances = pipeline
			.graph
			.iter_nodes()
			.map(|(name, x)| {
				(
					name.clone(),
					Arc::new(Mutex::new(x.build(ctx.clone(), name.into()))),
				)
			})
			.collect::<Vec<_>>();

		assert!(
			pipeline_inputs.len()
				== pipeline
					.graph
					.get_node(pipeline.input_node_idx)
					.1
					.inputs(ctx.clone())
					.len()
		);

		// The data inside each edge.
		// We consume node data once it is read so that unneeded memory may be freed.
		let mut edge_values = {
			pipeline
				.graph
				.iter_edges()
				.map(|_| EdgeValue::Uninitialized)
				.collect::<Vec<_>>()
		};

		// Keep track of nodes we have already run.
		// We already initialized all input edges, so mark that node `true`.
		let mut node_status = pipeline
			.graph
			.iter_nodes_idx()
			.map(|_| NodeRunState::NotRunning(PipelineNodeState::NotStarted))
			.collect::<Vec<_>>();

		// Threadpool we'll use to run nodes
		let pool = threadpool::Builder::new()
			.num_threads(self.node_runners)
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

		// Check every node.
		// TODO: write a smarter scheduler.
		loop {
			let mut finished_all_outputs = true;
			for (node, (_, _)) in pipeline.graph.iter_nodes_idx() {
				if !node_status[node.as_usize()].is_done() {
					finished_all_outputs = false;
				}

				self.try_run_node(
					ctx.clone(),
					&pipeline_inputs,
					node,
					&mut node_instances,
					pipeline.clone(),
					&pool,
					&mut node_status,
					&mut edge_values,
					send_data.clone(),
					send_status.clone(),
				)?;
			}

			// TODO: end condition.
			// TODO: after moves to END of pipeline node
			// TODO: handle all messages?
			// TODO: clean up threads?
			// TODO: quick node run, no thread

			for (node, port, data) in receive_data.try_iter() {
				// Fill every edge that is connected to
				// this output port of this node
				for edge_idx in pipeline
					.graph
					.edges_starting_at(node)
					.iter()
					.filter(|edge_idx| {
						let edge = &pipeline.graph.get_edge(**edge_idx).2;
						edge.source_port() == Some(port)
					}) {
					*edge_values.get_mut(edge_idx.as_usize()).unwrap() =
						EdgeValue::Data(data.clone());
				}
			}

			for (node, res) in receive_status.try_iter() {
				match res {
					Err(x) => {
						return Err(x);
					}
					Ok(status) => {
						*node_status.get_mut(node.as_usize()).unwrap() =
							NodeRunState::NotRunning(status);

						if status.is_done() {
							// When a node finishes successfully, mark all
							// `after` edges that start at it as "ready".
							for edge_idx in
								pipeline
									.graph
									.edges_starting_at(node)
									.iter()
									.filter(|edge_idx| {
										let edge = &pipeline.graph.get_edge(**edge_idx).2;
										edge.is_after()
									}) {
								*edge_values.get_mut(edge_idx.as_usize()).unwrap() =
									EdgeValue::AfterReady;
							}
						}
					}
				}
			}

			if finished_all_outputs {
				return Ok(());
			}
		}
	}

	/// Helper function, written here only for convenience.
	/// Try to run the node with index `n`.
	///
	/// Returns `Some(x)` if we ran the final output node,
	/// and `None` otherwise. All errors are sent to `txc`.
	#[inline]
	fn try_run_node(
		&mut self,
		ctx: Arc<<StubType::NodeType as PipelineNode>::NodeContext>,
		pipeline_inputs: &Vec<<StubType::NodeType as PipelineNode>::DataType>,
		node: GraphNodeIdx,
		node_instances: &mut Vec<(PipelineNodeLabel, Arc<Mutex<StubType::NodeType>>)>,
		pipeline: Arc<Pipeline<StubType>>,
		pool: &ThreadPool,
		node_status: &mut [NodeRunState],
		edge_values: &mut [EdgeValue<<StubType::NodeType as PipelineNode>::DataType>],
		send_data: Sender<(
			GraphNodeIdx,
			usize,
			<StubType::NodeType as PipelineNode>::DataType,
		)>,
		send_status: Sender<(GraphNodeIdx, Result<PipelineNodeState, PipelineError>)>,
	) -> Result<(), PipelineError> {
		// Skip nodes we've already run and nodes that are running right now.

		let n = node_status.get(node.as_usize()).unwrap();
		if n.is_running() || n.is_done() {
			return Ok(());
		}

		// Skip nodes we can't run
		if pipeline.graph.edges_ending_at(node).iter().any(|edge_idx| {
			match edge_values.get(edge_idx.as_usize()).unwrap() {
				// Any input edges uninitialized => This node hasn't been run yet, and is waiting on another.
				EdgeValue::Uninitialized => true,
				// All edges have data => good to go!
				EdgeValue::Data(_) => false,
				// All `after` edges are ready => good to go!
				EdgeValue::AfterReady => false,
				// No edges should be consumed unless this node has been started
				EdgeValue::Consumed => {
					if !n.is_pending() {
						let n = pipeline.graph.get_node(node);
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
			if node == pipeline.input_node_idx {
				pipeline_inputs.clone()
			} else {
				// Initialize all with None, in case some are disconnected.
				let node_type = &pipeline.graph.get_node(node).1;
				let mut inputs = Vec::with_capacity(node_type.inputs(ctx.clone()).len());
				for (_, t) in node_type.inputs(ctx.clone()).iter() {
					inputs.push(PipelineData::new_empty(t));
				}

				// Now, fill input values
				for edge_idx in pipeline.graph.edges_ending_at(node) {
					let edge = &pipeline.graph.get_edge(*edge_idx).2;

					// Skip non-value-carrying edges
					if !edge.is_ptp() {
						continue;
					}

					let val = edge_values.get_mut(edge_idx.as_usize()).unwrap();
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

		let (n, node_instance) = &node_instances.get(node.as_usize()).unwrap();
		let node_instance = node_instance.clone();
		let n = n.clone();

		// We MUST handle all status codes before re-running a node.
		// TODO: clean up scheduler

		// Initialize this node if we need to
		if node_status
			.get_mut(node.as_usize())
			.unwrap()
			.is_notstarted()
		{
			println!("Init {}", n);
			let mut node_instance_locked = node_instance.lock().unwrap();
			*node_status.get_mut(node.as_usize()).unwrap() = NodeRunState::Running;
			let res = node_instance_locked.init(ctx.clone(), prepare_inputs(), |port, data| {
				// This should never fail, since we never close the receiver.
				send_data.send((node, port, data)).unwrap();
				Ok(())
			});
			let done = res
				.as_ref()
				.ok()
				.map(|x| *x == PipelineNodeState::Done)
				.unwrap_or(true);
			send_status.send((node, res)).unwrap();

			// We don't need to run nodes that finished early
			if done {
				return Ok(());
			}
		} else {
			*node_status.get_mut(node.as_usize()).unwrap() = NodeRunState::Running;

			pool.execute(move || {
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
}
