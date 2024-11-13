use async_trait::async_trait;
use copper_util::graph::util::GraphNodeIdx;
use smartstring::{LazyCompact, SmartString};
use std::collections::BTreeMap;

use super::{NodeId, PortName, RunNodeError};
use crate::{data::PipeData, helpers::NodeParameters, CopperContext};

#[derive(Clone)]
pub struct ThisNodeInfo {
	pub idx: GraphNodeIdx,
	pub id: NodeId,
	pub node_type: SmartString<LazyCompact>,
}

#[async_trait]
pub trait Node<'ctx>: Sync + Send {
	/// Run this node. TODO: document
	async fn run(
		&self,
		ctx: &CopperContext<'ctx>,
		this_node: ThisNodeInfo,
		params: NodeParameters,
		input: BTreeMap<PortName, Option<PipeData>>,
	) -> Result<BTreeMap<PortName, PipeData>, RunNodeError>;
}
