use crossbeam::{
	channel::{unbounded, Receiver, Sender},
	select,
};
use std::{fmt::Debug, fs::File, io::Read, path::Path, sync::Arc};
use threadpool::ThreadPool;
use ufo_util::data::PipelineData;

use crate::{
	errors::PipelineError,
	nodes::nodeinstance::PipelineNodeInstance,
	syntax::{
		errors::PipelinePrepareError,
		labels::PipelineNodeLabel,
		spec::{PipelineConfig, PipelineSpec},
	},
	PipelineNode,
};

/// A specific port on a specific node
#[derive(Clone, Copy)]
pub(super) struct NodePort {
	pub node_idx: usize,
	pub port: usize,
}

impl Debug for NodePort {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		writeln!(f, "NodePort{{{}, {}}}", self.node_idx, self.port)
	}
}

/// An edge in a pipeline
#[derive(Debug)]
pub(super) enum PipelineEdge {
	/// A edge from an output port to an input port.
	/// PTP edges carry data between nodes.
	PortToPort((NodePort, NodePort)),

	/// An edge from a node to a node, specifying
	/// that the second *must* wait for the first.
	After((usize, usize)),
}

impl PipelineEdge {
	/// Is this a `Self::PortToPort`?
	pub fn is_ptp(&self) -> bool {
		matches!(self, Self::PortToPort(_))
	}

	/// Is this a `Self::After`?
	pub fn is_after(&self) -> bool {
		matches!(self, Self::After(_))
	}

	/// Get the node this edge starts at
	pub fn source_node(&self) -> usize {
		match self {
			Self::PortToPort((s, _)) => s.node_idx,
			Self::After((s, _)) => *s,
		}
	}

	/// Get the node this edge ends at
	pub fn target_node(&self) -> usize {
		match self {
			Self::PortToPort((_, t)) => t.node_idx,
			Self::After((_, t)) => *t,
		}
	}

	/// Get the port this edge starts at
	pub fn source_port(&self) -> Option<usize> {
		match self {
			Self::PortToPort((s, _)) => Some(s.port),
			Self::After(_) => None,
		}
	}

	/// Get the port this edge ends at
	pub fn target_port(&self) -> Option<usize> {
		match self {
			Self::PortToPort((_, t)) => Some(t.port),
			Self::After(_) => None,
		}
	}
}

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
pub struct Pipeline {
	/// This pipeline's configuration
	pub(crate) config: PipelineConfig,

	/// Array of nodes in this pipeline, indexed by node idx
	pub(crate) nodes: Arc<Vec<(PipelineNodeLabel, PipelineNodeInstance)>>,

	pub(crate) input_node_idx: usize,
	pub(crate) output_node_idx: usize,

	/// Array of directed edges, indexed by edge idx
	pub(crate) edges: Vec<PipelineEdge>,

	/// An array of edge idx, sorted by start node.
	pub(crate) edge_map_out: Vec<Vec<usize>>,

	/// An array of edge idx, sorted by end node.
	pub(crate) edge_map_in: Vec<Vec<usize>>,
}

impl Debug for Pipeline {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Pipeline")
			.field("nodes", &self.nodes)
			.field("edges", &self.edges)
			.finish()
	}
}

impl Pipeline {
	/// Try to load a pipeline from a file
	pub fn from_file(path: &Path) -> Result<Self, PipelinePrepareError> {
		let mut f =
			File::open(path).map_err(|error| PipelinePrepareError::CouldNotOpenFile { error })?;

		let mut s: String = Default::default();

		f.read_to_string(&mut s)
			.map_err(|error| PipelinePrepareError::CouldNotReadFile { error })?;

		let spec: PipelineSpec = toml::from_str(&s)
			.map_err(|error| PipelinePrepareError::CouldNotParseFile { error })?;

		spec.prepare()
	}
}

impl Pipeline {
	/// Get this pipeline's configuration
	pub fn get_config(&self) -> &PipelineConfig {
		&self.config
	}

	/// Run this pipeline using a maximum of `node_threads`
	/// workers to run nodes in parallel.
	pub fn run(
		&self,
		node_threads: usize,
		pipeline_inputs: Vec<PipelineData>,
	) -> Result<Vec<PipelineData>, PipelineError> {
		assert!(pipeline_inputs.len() == self.config.input.get_outputs().len());

		// The data inside each edge.
		// We consume node data once it is read so that unneeded memory may be freed.
		let mut edge_values = {
			(0..self.edges.len())
				.map(|edge_idx| {
					let edge = self.edges.get(edge_idx).unwrap();
					if edge.source_node() == self.input_node_idx {
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
		let mut node_has_been_run = (0..self.nodes.len())
			.map(|x| x == self.input_node_idx)
			.collect::<Vec<_>>();

		// Threadpool we'll use to run nodes
		let pool = threadpool::Builder::new()
			.num_threads(node_threads)
			.thread_name("Pipeline node runner".into())
			.build();

		// Channel for node data. Nodes send their outputs here once they are ready.
		//
		// Contents are (node index, port index, data)
		#[allow(clippy::type_complexity)]
		let (send_data, receive_data): (
			Sender<(usize, usize, PipelineData)>,
			Receiver<(usize, usize, PipelineData)>,
		) = unbounded();

		// Channel for node status. A node's return status is sent here when it finishes.
		//
		// Contents are (node index, result of `node.run()`)
		#[allow(clippy::type_complexity)]
		let (send_status, receive_status): (
			Sender<(usize, Result<(), PipelineError>)>,
			Receiver<(usize, Result<(), PipelineError>)>,
		) = unbounded();

		// Check every node.
		// TODO: write a smarter scheduler.
		loop {
			for n in 0..self.nodes.len() {
				if let Some(x) = Self::try_run_node(
					n,
					self,
					&pool,
					&mut node_has_been_run,
					&mut edge_values,
					send_data.clone(),
					send_status.clone(),
				) {
					return Ok(x);
				}
			}

			select! {
				recv(receive_data) -> msg => {
					let (node, port, data) = msg.unwrap();

					// Fill every edge that is connected to
					// this output port of this node
					for edge_idx in self
						.edge_map_out
						.get(node)
						.unwrap()
						.iter()
						.filter(|edge_idx| {
							let edge = self.edges.get(**edge_idx).unwrap();
							edge.source_port() == Some(port)
						})
					{
						*edge_values.get_mut(*edge_idx).unwrap() = EdgeValue::Data(data.clone());
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
							for edge_idx in self
								.edge_map_out
								.get(node)
								.unwrap()
								.iter()
								.filter(|edge_idx| {
									let edge = self.edges.get(**edge_idx).unwrap();
									edge.is_after()
								})
							{
								*edge_values.get_mut(*edge_idx).unwrap() = EdgeValue::AfterReady;
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
		n: usize,
		pipeline: &Pipeline,
		pool: &ThreadPool,
		node_has_been_run: &mut [bool],
		edge_values: &mut [EdgeValue],
		send_data: Sender<(usize, usize, PipelineData)>,
		send_status: Sender<(usize, Result<(), PipelineError>)>,
	) -> Option<Vec<PipelineData>> {
		// Skip nodes we've already run
		if *node_has_been_run.get(n).unwrap() {
			return None;
		}

		// Skip nodes we can't run
		if pipeline.edge_map_in.get(n).unwrap().iter().any(|edge_idx| {
			match edge_values.get(*edge_idx).unwrap() {
				// Any input edges uninitialized => This node hasn't been run yet, and is waiting on another.
				EdgeValue::Uninitialized => true,
				// All edges have data => good to go!
				EdgeValue::Data(_) => false,
				// All `after` edges are ready => good to go!
				EdgeValue::AfterReady => false,
				// Input edges are consumed when a node is run.
				// That case is handled earlier.
				EdgeValue::Consumed => unreachable!(),
			}
		}) {
			return None;
		}

		// We've found a node we can run, prepare inputs.
		let inputs = {
			// Initialize all with None, in case some are disconnected.
			let instance = &pipeline.nodes.get(n).unwrap().1;
			let mut inputs = Vec::with_capacity(instance.get_type().inputs().len());
			for (_, t) in instance.get_type().inputs().iter() {
				inputs.push(PipelineData::None(t));
			}

			// Now, fill input values
			for edge_idx in pipeline.edge_map_in.get(n).unwrap() {
				let edge = pipeline.edges.get(*edge_idx).unwrap();
				if !edge.is_ptp() {
					// Skip non-value-carrying edges
					continue;
				}

				let val = edge_values.get_mut(*edge_idx).unwrap();
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

		if n == pipeline.output_node_idx {
			// If we can run the pipeline output node, we're done.
			return Some(inputs);
		} else {
			// Otherwise, add this node to the pool.
			let pool_inputs = inputs.clone();
			let nodes = pipeline.nodes.clone();
			pool.execute(move || {
				let node = &nodes.get(n).unwrap().1;

				println!("Running {:?}", node);

				let res = node.run(
					|port, data| {
						// This should never fail, since we never close the receiver.
						send_data.send((n, port, data)).unwrap();
						Ok(())
					},
					pool_inputs,
				);

				send_status.send((n, res)).unwrap();
				println!("Done {:?}", node);
			});
			*node_has_been_run.get_mut(n).unwrap() = true;
		}

		return None;
	}
}
