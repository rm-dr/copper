use smartstring::{LazyCompact, SmartString};
use std::{collections::BTreeMap, marker::PhantomData};

use crate::{
	api::{
		NodeInputInfo, NodeOutputInfo, PipelineData, PipelineJobContext, PipelineNode,
		PipelineNodeError, PipelineNodeState,
	},
	dispatcher::NodeParameterValue,
	labels::PipelinePortID,
};

pub const INPUT_NODE_TYPE_NAME: &str = "Input";

pub struct Input<DataType: PipelineData, ContextType: PipelineJobContext<DataType>> {
	_p: PhantomData<ContextType>,

	outputs: [NodeOutputInfo<DataType::DataStubType>; 1],
	value: DataType,
}

impl<DataType: PipelineData, ContextType: PipelineJobContext<DataType>>
	Input<DataType, ContextType>
{
	pub fn new(
		ctx: &ContextType,
		params: &BTreeMap<SmartString<LazyCompact>, NodeParameterValue<DataType>>,
		node_name: &str,
	) -> Self {
		if params.len() != 1 {
			panic!()
		}

		let data_type = if let Some(value) = params.get("data_type") {
			match value {
				NodeParameterValue::DataType(data_type) => data_type.clone(),
				_ => panic!(),
			}
		} else {
			panic!()
		};

		let value = if let Some(value) = ctx.get_input().get(node_name) {
			value.clone()
		} else {
			// TODO: this is a hack, required because we must build nodes to get their stats.
			DataType::disconnected(data_type)
		};

		if data_type != value.as_stub() {
			panic!()
		}

		Self {
			_p: PhantomData {},
			outputs: [NodeOutputInfo {
				name: PipelinePortID::new("out"),
				produces_type: data_type,
			}],
			value,
		}
	}
}

impl<DataType: PipelineData, ContextType: PipelineJobContext<DataType>> PipelineNode<DataType>
	for Input<DataType, ContextType>
{
	fn inputs(&self) -> &[NodeInputInfo<DataType::DataStubType>] {
		&[]
	}

	fn outputs(&self) -> &[NodeOutputInfo<DataType::DataStubType>] {
		&self.outputs
	}

	fn quick_run(&self) -> bool {
		true
	}

	fn take_input(
		&mut self,
		_target_port: usize,
		_input_data: DataType,
	) -> Result<(), PipelineNodeError> {
		unreachable!()
	}

	fn run(
		&mut self,
		send_data: &dyn Fn(usize, DataType) -> Result<(), PipelineNodeError>,
	) -> Result<PipelineNodeState, PipelineNodeError> {
		send_data(0, self.value.clone())?;
		Ok(PipelineNodeState::Done)
	}
}
