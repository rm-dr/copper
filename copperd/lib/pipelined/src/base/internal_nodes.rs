use smartstring::{LazyCompact, SmartString};
use std::{collections::BTreeMap, marker::PhantomData};

use crate::base::{InitNodeError, Node, NodeState, PipelineData, RunNodeError};

use super::{NodeParameterValue, NodeSignal, PipelineJobContext, PortName, ProcessSignalError};

pub struct Input<DataType: PipelineData, ContextType: PipelineJobContext<DataType>> {
	_p: PhantomData<ContextType>,
	value: DataType,
}

impl<DataType: PipelineData, ContextType: PipelineJobContext<DataType>>
	Input<DataType, ContextType>
{
	pub fn new(
		ctx: &ContextType,
		params: &BTreeMap<SmartString<LazyCompact>, NodeParameterValue<DataType>>,
		node_name: &str,
	) -> Result<Self, InitNodeError> {
		if params.len() != 0 {
			return Err(InitNodeError::BadParameterCount { expected: 0 });
		}

		// TODO: input as parameter

		let value = if let Some(value) = ctx.get_input().get(node_name) {
			value.clone()
		} else {
			panic!();
			/*
			return Err(InitNodeError::MissingInput {
				input_name: node_name.into(),
			});
			*/
		};

		Ok(Self {
			_p: PhantomData {},
			value,
		})
	}
}

impl<DataType: PipelineData, ContextType: PipelineJobContext<DataType>> Node<DataType>
	for Input<DataType, ContextType>
{
	fn process_signal(&mut self, signal: NodeSignal<DataType>) -> Result<(), ProcessSignalError> {
		match signal {
			NodeSignal::ConnectInput { .. } => {
				return Err(ProcessSignalError::InputPortDoesntExist)
			}

			NodeSignal::DisconnectInput { .. } => {
				return Err(ProcessSignalError::InputPortDoesntExist)
			}

			NodeSignal::ReceiveInput { .. } => {
				return Err(ProcessSignalError::InputPortDoesntExist)
			}
		}
	}

	fn quick_run(&self) -> bool {
		true
	}

	fn run(
		&mut self,
		send_data: &dyn Fn(PortName, DataType) -> Result<(), RunNodeError>,
	) -> Result<NodeState, RunNodeError> {
		send_data(PortName::new("out"), self.value.clone())?;
		Ok(NodeState::Done)
	}
}
