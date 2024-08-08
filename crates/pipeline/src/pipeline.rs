use crossbeam::channel::{unbounded, Receiver, Sender};
use std::{
	fmt::Debug,
	fs::File,
	io::Read,
	path::Path,
	sync::{Arc, Mutex},
};
use threadpool::ThreadPool;
use ufo_util::data::PipelineData;

use crate::{
	errors::PipelineError,
	nodes::{nodeinstance::PipelineNodeInstance, nodetype::PipelineNodeType},
	syntax::{
		errors::PipelinePrepareError,
		labels::PipelineNodeLabel,
		spec::{PipelineConfig, PipelineSpec},
	},
	PipelineStatelessRunner,
};

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

#[derive(Debug)]
enum EdgeValue {
	Uninitialized,
	Data(Arc<PipelineData>),
	Consumed,
}

impl EdgeValue {
	fn unwrap(self) -> Arc<PipelineData> {
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
	pub(crate) nodes: Vec<(PipelineNodeLabel, PipelineNodeType)>,

	/// The index of this pipeline's external node in [`Self::nodes`]
	pub(crate) external_node_idx: usize,

	/// Array of directed edges, indexed by edge idx
	pub(crate) edges: Vec<(NodePort, NodePort)>,

	/// An array of edge idx, sorted by start node.
	pub(crate) edge_map_out: Vec<Vec<usize>>,
	/// edge_map, but reversed
	pub(crate) edge_map_in: Vec<Vec<usize>>,
}

impl Debug for Pipeline {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Pipeline")
			.field("nodes", &self.nodes)
			.field("external_node_idx", &self.external_node_idx)
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
		inputs: Vec<Arc<PipelineData>>,
	) -> Result<Vec<Arc<PipelineData>>, PipelineError> {
		assert!(inputs.len() == self.config.input.get_outputs().len());

		// Create node instances for this run
		let node_instances = Arc::new(
			self.nodes
				.iter()
				.map(|x| Mutex::new(x.1.build((&x.0).into())))
				.collect::<Vec<_>>(),
		);

		// The data inside each edge.
		// We consume node data once it is read so that unneeded memory may be freed.
		let mut edge_values = {
			let mut values = (0..self.edges.len())
				.map(|_| EdgeValue::Uninitialized)
				.collect::<Vec<_>>();

			// Place initial inputs
			for edge_idx in self.edge_map_out.get(self.external_node_idx).unwrap() {
				let edge = self.edges.get(*edge_idx).unwrap();
				values[*edge_idx] = EdgeValue::Data(inputs.get(edge.0.port).unwrap().clone());
			}
			values
		};

		// Keep track of nodes we have already run
		let mut node_has_been_run = (0..self.nodes.len()).map(|_| false).collect::<Vec<_>>();

		// Threadpool we'll use to run nodes
		let pool = ThreadPool::new(node_threads);

		// Channel for node data. Nodes send their outputs here once they are ready.
		let (send_data, receive_data): (
			// (node index, result of `node.run()`)
			Sender<(usize, Result<Vec<Arc<PipelineData>>, PipelineError>)>,
			Receiver<(usize, Result<Vec<Arc<PipelineData>>, PipelineError>)>,
		) = unbounded();

		// Check every node.
		// The fancy iterator makes sure that the external node is checked first.
		// If we can run it, we're done!
		// TODO: write a smarter scheduler.
		loop {
			for n in std::iter::once(self.external_node_idx)
				.chain((0..self.nodes.len()).filter(|x| *x != self.external_node_idx))
			{
				if let Some(x) = Self::try_run_node(
					n,
					self,
					&pool,
					&mut node_has_been_run,
					&mut edge_values,
					node_instances.clone(),
					send_data.clone(),
				) {
					return Ok(x);
				}
			}

			let (n, out) = receive_data.recv().unwrap();
			let out = out?;

			for edge_idx in self.edge_map_out.get(n).unwrap() {
				let edge = self.edges.get(*edge_idx).unwrap();
				*edge_values.get_mut(*edge_idx).unwrap() =
					EdgeValue::Data(out.get(edge.0.port).unwrap().clone());
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
		node_has_been_run: &mut Vec<bool>,
		edge_values: &mut Vec<EdgeValue>,
		node_instances: Arc<Vec<Mutex<PipelineNodeInstance>>>,
		send_data: Sender<(usize, Result<Vec<Arc<PipelineData>>, PipelineError>)>,
	) -> Option<Vec<Arc<PipelineData>>> {
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
			let mut inputs = match &*node_instances.get(n).unwrap().lock().unwrap() {
				PipelineNodeInstance::ConstantNode(_) => {
					vec![]
				}
				PipelineNodeInstance::ExternalNode => {
					let mut inputs = Vec::with_capacity(pipeline.config.output.get_inputs().len());
					for (_, t) in pipeline.config.output.get_inputs().iter() {
						inputs.push(Arc::new(PipelineData::None(t)));
					}
					inputs
				}
				x => {
					let mut inputs = Vec::with_capacity(x.inputs().unwrap().len());
					for (_, t) in x.inputs().unwrap().iter() {
						inputs.push(Arc::new(PipelineData::None(t)));
					}
					inputs
				}
			};

			// Now, fill input values
			for edge_idx in pipeline.edge_map_in.get(n).unwrap() {
				let edge = pipeline.edges.get(*edge_idx).unwrap();
				let val = edge_values.get_mut(*edge_idx).unwrap();
				match val {
					EdgeValue::Data(_) => {
						let x = std::mem::replace(val, EdgeValue::Consumed);
						*inputs.get_mut(edge.1.port).unwrap() = x.unwrap();
					}
					_ => unreachable!(),
				};
			}

			inputs
		};

		if n == pipeline.external_node_idx {
			// If we can run the external node, we're done.
			return Some(inputs);
		} else {
			// Otherwise, add this node to the pool.
			let pool_inputs = inputs.clone();
			pool.execute(move || {
				let node = node_instances.get(n).unwrap().lock().unwrap();
				println!("Running {:?}", node);
				// TODO: remove (debug)
				std::thread::sleep(std::time::Duration::from_secs(2));

				// TODO: have nodes send data whenever it's ready
				send_data.send((n, node.run(pool_inputs))).unwrap();
				println!("Done {:?}", node);
			});
			*node_has_been_run.get_mut(n).unwrap() = true;
		}

		return None;
	}
}
