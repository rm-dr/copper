use crossbeam::{
	channel::{unbounded, Receiver, Sender},
	select,
};
use futures::executor::block_on;
use smartstring::{LazyCompact, SmartString};
use std::{
	fmt::Debug,
	fs::File,
	io::Read,
	path::Path,
	sync::{Arc, Mutex},
};
use threadpool::ThreadPool;
use ufo_storage::{api::Dataset, sea::dataset::SeaDataset};
use ufo_util::{data::PipelineData, graph::GraphNodeIdx};

use crate::{
	errors::PipelineError,
	nodes::{nodeinstance::PipelineNodeInstance, nodetype::PipelineNodeType},
	output::{storage::StorageOutput, PipelineOutput, PipelineOutputKind},
	pipeline::Pipeline,
	syntax::{errors::PipelinePrepareError, labels::PipelineNodeLabel, spec::PipelineSpec},
	PipelineNode,
};

#[derive(Debug)]
enum EdgeValue {
	/// This edge is waiting on another node to run
	Uninitialized,

	/// This edge has data that is ready to be used
	/// (Only valid for Edge::PortToPort)
	Data(PipelineData),

	/// This edge had data, but it has been consumed
	/// (Only valid for Edge::PortToPort)
	Consumed,

	/// This edge's source node has finised running
	/// (Only valid for Edge::After)
	AfterReady,
}

impl EdgeValue {
	fn unwrap(self) -> PipelineData {
		match self {
			Self::Data(x) => x,
			_ => panic!("tried to unwrap a non-Data Edgevalue"),
		}
	}
}

/// A prepared data processing pipeline.
/// This is guaranteed to be correct:
/// no dependency cycles, no port type mismatch, etc
pub struct PipelineRunner {
	dataset: SeaDataset,
	pipelines: Vec<(SmartString<LazyCompact>, Arc<Pipeline>)>,

	node_runners: usize,
}

impl PipelineRunner {
	pub fn new(dataset: SeaDataset, node_runners: usize) -> Self {
		Self {
			dataset,
			pipelines: Vec::new(),
			node_runners,
		}
	}

	pub fn add_pipeline(
		&mut self,
		path: &Path,
		pipeline_name: String,
	) -> Result<(), PipelinePrepareError> {
		let mut f =
			File::open(path).map_err(|error| PipelinePrepareError::CouldNotOpenFile { error })?;

		let mut s: String = Default::default();

		f.read_to_string(&mut s)
			.map_err(|error| PipelinePrepareError::CouldNotReadFile { error })?;

		let spec: PipelineSpec = toml::from_str(&s)
			.map_err(|error| PipelinePrepareError::CouldNotParseFile { error })?;

		let p = spec.prepare(pipeline_name.clone(), &self.pipelines)?;
		self.pipelines.push((pipeline_name.into(), Arc::new(p)));
		return Ok(());
	}

	pub fn get_pipeline(&self, pipeline_name: SmartString<LazyCompact>) -> Option<Arc<Pipeline>> {
		self.pipelines
			.iter()
			.find(|(x, _)| x == &pipeline_name)
			.map(|(_, x)| x.clone())
	}
}

impl PipelineRunner {
	/// Run a pipeline to completion.
	pub fn run(
		&mut self,
		pipeline_name: SmartString<LazyCompact>,
		pipeline_inputs: Vec<PipelineData>,
	) -> Result<(), PipelineError> {
		let pipeline = self.get_pipeline(pipeline_name).unwrap();

		// TODO: async-like scheduler with node state
		let mut node_instances = pipeline
			.graph
			.iter_nodes()
			.map(|(name, x)| (name.clone(), Arc::new(Mutex::new(x.build(name.into())))))
			.collect::<Vec<_>>();

		// The index of this pipeline's input node
		// (we are guaranteed to have exactly one)
		let input_node_idx = {
			pipeline
				.graph
				.iter_nodes_idx()
				.find(|(_, (_, n))| n.is_pipeline_input())
				.map(|(i, _)| i)
				.unwrap()
		};

		assert!(pipeline_inputs.len() == pipeline.config.input.get_outputs().len());

		// The data inside each edge.
		// We consume node data once it is read so that unneeded memory may be freed.
		let mut edge_values = {
			pipeline
				.graph
				.iter_edges()
				.map(|(f, _, edge)| {
					if f == &input_node_idx {
						EdgeValue::Data(
							pipeline_inputs
								.get(edge.source_port().unwrap())
								.unwrap()
								.clone(),
						)
					} else {
						EdgeValue::Uninitialized
					}
				})
				.collect::<Vec<_>>()
		};

		// Keep track of nodes we have already run.
		// We already initialized all input edges, so mark that node `true`.
		let mut node_has_been_run = pipeline
			.graph
			.iter_nodes_idx()
			.map(|(x, _)| x == input_node_idx)
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
			Sender<(GraphNodeIdx, usize, PipelineData)>,
			Receiver<(GraphNodeIdx, usize, PipelineData)>,
		) = unbounded();

		// Channel for node status. A node's return status is sent here when it finishes.
		//
		// Contents are (node index, result of `node.run()`)
		#[allow(clippy::type_complexity)]
		let (send_status, receive_status): (
			Sender<(GraphNodeIdx, Result<(), PipelineError>)>,
			Receiver<(GraphNodeIdx, Result<(), PipelineError>)>,
		) = unbounded();

		// Check every node.
		// TODO: write a smarter scheduler.
		loop {
			for (node, _) in pipeline.graph.iter_nodes_idx() {
				if let Some((name, outputs)) = self.try_run_node(
					node,
					&mut node_instances,
					pipeline.clone(),
					&pool,
					&mut node_has_been_run,
					&mut edge_values,
					send_data.clone(),
					send_status.clone(),
				) {
					let p = self.get_pipeline(name.into()).unwrap();
					self.finish_pipeline(p.clone(), outputs)?;
					if p.name == pipeline.name {
						return Ok(());
					}
				}
			}

			select! {
				recv(receive_data) -> msg => {
					let (node, port, data) = msg.unwrap();

					// Fill every edge that is connected to
					// this output port of this node
					for edge_idx in pipeline
						.graph
						.edges_starting_at(node)
						.iter()
						.filter(|edge_idx| {
							let edge = &pipeline.graph.get_edge(**edge_idx).2;
							edge.source_port() == Some(port)
						})
					{
						*edge_values.get_mut(edge_idx.as_usize()).unwrap() = EdgeValue::Data(data.clone());
					}
				}

				recv(receive_status) -> msg => {
					match msg.unwrap() {
						(_node, Err(x)) => {
							return Err(x);
						},
						(node, Ok(_)) => {

							// When a node finishes successfully, mark all
							// `after` edges that start at it as "ready".
							for edge_idx in pipeline
								.graph
								.edges_starting_at(node)
								.iter()
								.filter(|edge_idx| {
									let edge = &pipeline.graph.get_edge(**edge_idx).2;
									edge.is_after()
								})
							{
								*edge_values
									.get_mut(edge_idx.as_usize())
									.unwrap() = EdgeValue::AfterReady;
							}
						}
					}
				}
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
		node: GraphNodeIdx,
		node_instances: &mut Vec<(PipelineNodeLabel, Arc<Mutex<PipelineNodeInstance>>)>,
		pipeline: Arc<Pipeline>,
		pool: &ThreadPool,
		node_has_been_run: &mut [bool],
		edge_values: &mut [EdgeValue],
		send_data: Sender<(GraphNodeIdx, usize, PipelineData)>,
		send_status: Sender<(GraphNodeIdx, Result<(), PipelineError>)>,
	) -> Option<(String, Vec<PipelineData>)> {
		// Skip nodes we've already run
		if *node_has_been_run.get(node.as_usize()).unwrap() {
			return None;
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
				// Input edges are consumed when a node is run.
				// That case is handled earlier.
				EdgeValue::Consumed => unreachable!("tried to use consumed edge"),
			}
		}) {
			return None;
		}

		// We've found a node we can run, prepare inputs.
		let inputs = {
			// Initialize all with None, in case some are disconnected.
			let node_type = &pipeline.graph.get_node(node).1;
			let mut inputs = Vec::with_capacity(node_type.inputs().len());
			for (_, t) in node_type.inputs().iter() {
				inputs.push(PipelineData::None(t));
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
		};

		match &pipeline.graph.get_node(node).1 {
			PipelineNodeType::PipelineOutputs { pipeline, .. } => {
				return Some((pipeline.clone(), inputs));
			}

			// Otherwise, add this node to the pool.
			_ => {
				let pool_inputs = inputs.clone();
				let (n, node_instance) = &node_instances.get(node.as_usize()).unwrap();
				let node_instance = node_instance.clone();
				let n = n.clone();

				pool.execute(move || {
					let node_instance = node_instance.lock().unwrap();

					println!("Running {}", n);

					let res = node_instance.run(
						|port, data| {
							// This should never fail, since we never close the receiver.
							send_data.send((node, port, data)).unwrap();
							Ok(())
						},
						pool_inputs,
					);

					send_status.send((node, res)).unwrap();
					println!("Done {}", n);
				});
				*node_has_been_run.get_mut(node.as_usize()).unwrap() = true;
			}
		}

		return None;
	}
}

impl PipelineRunner {
	fn finish_pipeline(
		&mut self,
		pipeline: Arc<Pipeline>,
		outputs: Vec<PipelineData>,
	) -> Result<(), PipelineError> {
		match &pipeline.config.output {
			PipelineOutputKind::DataSet { attrs, class } => {
				let c = block_on(self.dataset.get_class(&class[..]))
					.unwrap()
					.unwrap();
				let mut e = StorageOutput::new(
					&mut self.dataset,
					c,
					attrs.iter().map(|(a, b)| (a.into(), *b)).collect(),
				);
				e.run(outputs.iter().collect()).unwrap();
			}
		}

		return Ok(());
	}
}
