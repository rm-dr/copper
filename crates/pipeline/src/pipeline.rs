use std::{fmt::Debug, sync::Arc};

use crate::{
	data::PipelineData, errors::PipelineError, nodes::PipelineNodeInstance,
	syntax::spec::PipelineConfig, PipelineStatelessRunner,
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
	Data(Option<Arc<PipelineData>>),
	Consumed,
}

/// A prepared data processing pipeline
pub struct Pipeline {
	/// This pipeline's configuration
	pub(crate) config: PipelineConfig,

	/// Array of nodes in this pipeline, indexed by node idx
	pub(crate) nodes: Vec<PipelineNodeInstance>,

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
	pub fn get_config(&self) -> &PipelineConfig {
		&self.config
	}

	pub fn run(
		&self,
		inputs: Vec<Option<Arc<PipelineData>>>,
	) -> Result<Vec<Option<Arc<PipelineData>>>, PipelineError> {
		// The data inside each edge.
		// We consume node data once it is read so that unneeded memory may be freed.
		let mut edge_values = (0..self.edges.len())
			.map(|_| EdgeValue::Uninitialized)
			.collect::<Vec<_>>();

		// Keep track of nodes that have been run
		let mut node_has_been_run = (0..self.nodes.len()).map(|_| false).collect::<Vec<_>>();

		// Place initial inputs
		for edge_idx in self.edge_map_out.get(self.external_node_idx).unwrap() {
			let edge = self.edges.get(*edge_idx).unwrap();
			edge_values[*edge_idx] = EdgeValue::Data(inputs.get(edge.0.port).unwrap().clone());
		}

		let mut inputs = Vec::new();
		loop {
			for n in 0..self.nodes.len() {
				// Skip nodes we've already run
				if *node_has_been_run.get(n).unwrap() {
					continue;
				}

				// Skip nodes we can't run
				if self.edge_map_in.get(n).unwrap().iter().any(|edge_idx| {
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
					continue;
				}

				// We've found a node we can run.
				// Prepare inputs.
				inputs.clear();

				let n_inputs = match self.nodes.get(n).unwrap() {
					PipelineNodeInstance::ConstantNode(_) => 0,
					PipelineNodeInstance::ExternalNode => self.config.output.get_inputs().len(),
					x => x.get_type().unwrap().n_inputs(),
				};

				// Initialize all inputs with None,
				// in case some are disconnected.
				for _ in 0..n_inputs {
					inputs.push(None);
				}

				// Fill input values
				for edge_idx in self.edge_map_in.get(n).unwrap().iter() {
					let edge = self.edges.get(*edge_idx).unwrap();
					match edge_values.get(*edge_idx).unwrap() {
						EdgeValue::Data(x) => *inputs.get_mut(edge.1.port).unwrap() = x.clone(),
						_ => unreachable!(),
					};
				}

				if n == self.external_node_idx {
					// If we can run the external node, we're done.
					return Ok(inputs);
				} else {
					// We ran an intermediate node, fill in output edges and consume input edges
					let out = self.nodes.get(n).unwrap().run(inputs.clone())?;
					*node_has_been_run.get_mut(n).unwrap() = true;
					for edge_idx in self.edge_map_out.get(n).unwrap() {
						let edge = self.edges.get(*edge_idx).unwrap();
						*edge_values.get_mut(*edge_idx).unwrap() =
							EdgeValue::Data(out.get(edge.0.port).unwrap().clone());
					}
					for edge_idx in self.edge_map_in.get(n).unwrap() {
						*edge_values.get_mut(*edge_idx).unwrap() = EdgeValue::Consumed
					}
				}
			}
		}
	}
}
