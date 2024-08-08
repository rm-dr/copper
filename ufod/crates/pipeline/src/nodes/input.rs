use smartstring::{LazyCompact, SmartString};
use std::{collections::BTreeMap, marker::PhantomData};

use crate::{
	api::{
		InitNodeError, NodeInfo, PipelineData, PipelineJobContext, PipelineNode, PipelineNodeState,
		RunNodeError,
	},
	dispatcher::NodeParameterValue,
	labels::PipelinePortID,
};

pub const INPUT_NODE_TYPE_NAME: &str = "Input";

pub struct InputInfo<DataType: PipelineData> {
	outputs: [(PipelinePortID, DataType::DataStubType); 1],
	data_type: DataType::DataStubType,
}

impl<DataType: PipelineData> InputInfo<DataType> {
	pub fn new(
		params: &BTreeMap<SmartString<LazyCompact>, NodeParameterValue<DataType>>,
		_node_name: &str,
	) -> Result<Self, InitNodeError> {
		if params.len() != 1 {
			return Err(InitNodeError::BadParameterCount { expected: 1 });
		}

		let data_type = if let Some(value) = params.get("data_type") {
			match value {
				NodeParameterValue::DataType(data_type) => data_type.clone(),
				_ => {
					return Err(InitNodeError::BadParameterType {
						param_name: "data_type".into(),
					});
				}
			}
		} else {
			return Err(InitNodeError::MissingParameter {
				param_name: "data_type".into(),
			});
		};

		Ok(Self {
			outputs: [(PipelinePortID::new("out"), data_type)],
			data_type,
		})
	}
}

impl<DataType: PipelineData> NodeInfo<DataType> for InputInfo<DataType> {
	fn inputs(&self) -> &[(PipelinePortID, <DataType as PipelineData>::DataStubType)] {
		&[]
	}

	fn outputs(&self) -> &[(PipelinePortID, <DataType as PipelineData>::DataStubType)] {
		&self.outputs
	}
}

pub struct Input<DataType: PipelineData, ContextType: PipelineJobContext<DataType>> {
	_p: PhantomData<ContextType>,
	info: InputInfo<DataType>,
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
		let info = InputInfo::new(params, node_name)?;

		let value = if let Some(value) = ctx.get_input().get(node_name) {
			value.clone()
		} else {
			// TODO: this is a hack, required because we must build nodes to get their stats.
			DataType::disconnected(info.data_type)
		};

		if info.data_type != value.as_stub() {
			return Err(InitNodeError::BadInputType);
		}

		Ok(Self {
			_p: PhantomData {},
			info,
			value,
		})
	}
}

impl<DataType: PipelineData, ContextType: PipelineJobContext<DataType>> PipelineNode<DataType>
	for Input<DataType, ContextType>
{
	fn get_info(&self) -> &dyn NodeInfo<DataType> {
		&self.info
	}

	fn quick_run(&self) -> bool {
		true
	}

	fn take_input(
		&mut self,
		_target_port: usize,
		_input_data: DataType,
	) -> Result<(), RunNodeError> {
		unreachable!("Input nodes do not have any input ports.")
	}

	fn run(
		&mut self,
		send_data: &dyn Fn(usize, DataType) -> Result<(), RunNodeError>,
	) -> Result<PipelineNodeState, RunNodeError> {
		send_data(0, self.value.clone())?;
		Ok(PipelineNodeState::Done)
	}
}
