use async_trait::async_trait;
use smartstring::{LazyCompact, SmartString};
use std::collections::BTreeMap;

use super::{NodeParameterValue, PipelineData, PipelineJobContext, PortName, RunNodeError};

#[async_trait]
pub trait Node<DataType: PipelineData, ContextType: PipelineJobContext>: Sync + Send {
	/// Run this node. TODO: document streams
	/// and blocking tasks
	async fn run(
		&self,
		ctx: &ContextType,
		params: BTreeMap<SmartString<LazyCompact>, NodeParameterValue<DataType>>,
		input: BTreeMap<PortName, DataType>,
	) -> Result<BTreeMap<PortName, DataType>, RunNodeError>;
}
