//! Helper structs for node inputs and outputs

use serde::Deserialize;
use std::{fmt::Debug, str::FromStr};

use crate::labels::{PipelineNodeID, PipelinePortID};

/// An output port in the pipeline.
/// (i.e, a port that produces data.)
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct NodeOutput {
	/// The node that provides this output
	pub node: PipelineNodeID,

	/// The output's name
	pub port: PipelinePortID,
}

impl FromStr for NodeOutput {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let mut i = s.split('.');
		let a = i.next();
		let b = i.next();

		if a.is_none() || b.is_none() || i.next().is_some() {
			return Err(());
		}
		let a = a.unwrap();
		let b = b.unwrap();

		Ok(Self {
			node: PipelineNodeID::new(a),
			port: PipelinePortID::new(b),
		})
	}
}

/// An input port in the pipeline.
/// (i.e, a port that consumes data.)
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct NodeInput {
	/// The node that provides this input
	pub node: PipelineNodeID,

	/// The port's name
	pub port: PipelinePortID,
}

impl FromStr for NodeInput {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let mut i = s.split('.');
		let a = i.next();
		let b = i.next();

		if a.is_none() || b.is_none() || i.next().is_some() {
			return Err(());
		}
		let a = a.unwrap();
		let b = b.unwrap();

		Ok(Self {
			node: PipelineNodeID::new(a),
			port: PipelinePortID::new(b),
		})
	}
}
