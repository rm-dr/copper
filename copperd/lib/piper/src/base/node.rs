use async_trait::async_trait;
use copper_util::graph::util::GraphNodeIdx;
use smartstring::{LazyCompact, SmartString};
use std::collections::BTreeMap;
use tokio::sync::mpsc;

use super::{
	NodeId, NodeParameterValue, PipelineData, PipelineJobContext, PipelineJobResult, PortName,
	RunNodeError,
};

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
pub trait Node<
	ResultType: PipelineJobResult,
	DataType: PipelineData,
	ContextType: PipelineJobContext<DataType, ResultType>,
>: Sync + Send
{
	/// Run this node. TODO: document
	async fn run(
		&self,
		ctx: &ContextType,
		this_node: ThisNodeInfo,
		params: BTreeMap<SmartString<LazyCompact>, NodeParameterValue>,
		input: BTreeMap<PortName, Option<DataType>>,
		output: mpsc::Sender<NodeOutput<DataType>>,
	) -> Result<(), RunNodeError<DataType>>;
}
