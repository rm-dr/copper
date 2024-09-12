use async_trait::async_trait;
use smartstring::{LazyCompact, SmartString};
use std::collections::BTreeMap;
use tokio::sync::oneshot;

use super::{NodeParameterValue, PipelineData, PipelineJobContext, PortName, RunNodeError};

#[derive(Debug)]
pub enum NodeOutput<DataType: PipelineData> {
	Plain(Option<DataType>),
	Awaited(oneshot::Receiver<Result<Option<DataType>, RunNodeError>>),
}

impl<DataType: PipelineData> NodeOutput<DataType> {
	pub async fn get_value(self) -> Result<Option<DataType>, RunNodeError> {
		match self {
			Self::Plain(x) => Ok(x),
			Self::Awaited(x) => Ok(x.await??),
		}
	}

	/// A hacky alternative to .clone()
	///
	/// We can't clone a [`oneshot::Receiver`], so we can't clone a [`NodeOutput`].
	/// This is the next best thing. (It's actually pretty clever!)
	///
	/// We need to be able to duplicate `NodeOutputs` because one node's output might
	/// be connected to many edges. Each of those edges needs an owned [`NodeOutput`]
	/// that produces the same data.
	pub fn dupe(self) -> (Self, Self) {
		match self {
			// This we can just clone...
			Self::Plain(ref x) => (Self::Plain(x.clone()), self),

			// But this can't be cloned.
			// Instead, spawn a task that clones the result when it's ready
			// and sends it to two new `Self::Awaited`s.
			Self::Awaited(x) => {
				let (txa, rxa) = oneshot::channel();
				let (txb, rxb) = oneshot::channel();
				tokio::spawn(async {
					let r: Result<Option<DataType>, RunNodeError> = match x.await {
						Ok(x) => x,
						Err(x) => Err(x.into()),
					};

					// We don't care if these fail.
					// Errors only occur if the corresponding
					// receiver has been dropped.
					let _ = txa.send(r.clone());
					let _ = txb.send(r);
				});

				(Self::Awaited(rxa), Self::Awaited(rxb))
			}
		}
	}
}

#[async_trait]
pub trait Node<DataType: PipelineData, ContextType: PipelineJobContext>: Sync + Send {
	/// Run this node. TODO: document
	///
	/// if run() cannot produce all its output imediately, it should tokio::spawn a task.
	async fn run(
		&self,
		ctx: &ContextType,
		params: BTreeMap<SmartString<LazyCompact>, NodeParameterValue<DataType>>,
		input: BTreeMap<PortName, NodeOutput<DataType>>,
	) -> Result<BTreeMap<PortName, NodeOutput<DataType>>, RunNodeError>;
}
