use async_trait::async_trait;
use copper_util::graph::util::GraphNodeIdx;
use smartstring::{LazyCompact, SmartString};
use std::collections::BTreeMap;
use tokio::sync::mpsc;

use super::{NodeId, NodeParameterValue, PipelineData, PipelineJobContext, PortName, RunNodeError};

#[derive(Clone)]
pub struct ThisNodeInfo {
	pub idx: GraphNodeIdx,
	pub id: NodeId,
	pub node_type: SmartString<LazyCompact>,
}

#[derive(Clone)]
pub struct NodeOutput<DataType: PipelineData> {
	pub node: ThisNodeInfo,
	pub port: PortName,
	pub data: Option<DataType>,
}

#[async_trait]
pub trait Node<DataType: PipelineData, ContextType: PipelineJobContext>: Sync + Send {
	/// Run this node. TODO: document
	async fn run(
		&self,
		ctx: &ContextType,
		this_node: ThisNodeInfo,
		params: BTreeMap<SmartString<LazyCompact>, NodeParameterValue<DataType>>,
		input: BTreeMap<PortName, Option<DataType>>,
		output: mpsc::Sender<NodeOutput<DataType>>,
	) -> Result<(), RunNodeError<DataType>>;
}
