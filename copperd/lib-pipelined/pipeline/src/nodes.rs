use smartstring::{LazyCompact, SmartString};
use std::{collections::BTreeMap, marker::PhantomData};

use crate::{
	base::{
		InitNodeError, Node, NodeInfo, NodeState, PipelineData, PipelineJobContext, RunNodeError,
	},
	dispatcher::NodeParameterValue,
	labels::PipelinePortID,
};

pub const INPUT_NODE_TYPE_NAME: &str = "Input";

pub struct InputInfo<DataType: PipelineData> {
	inputs: BTreeMap<PipelinePortID, DataType::DataStubType>,
	outputs: BTreeMap<PipelinePortID, DataType::DataStubType>,
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
				NodeParameterValue::DataType(data_type) => *data_type,
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
			inputs: BTreeMap::new(),
			outputs: BTreeMap::from([(PipelinePortID::new("out"), data_type)]),
			data_type,
		})
	}
}

impl<DataType: PipelineData> NodeInfo<DataType> for InputInfo<DataType> {
	fn inputs(&self) -> &BTreeMap<PipelinePortID, <DataType as PipelineData>::DataStubType> {
		&self.inputs
	}

	fn outputs(&self) -> &BTreeMap<PipelinePortID, <DataType as PipelineData>::DataStubType> {
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
			return Err(InitNodeError::MissingInput {
				input_name: node_name.into(),
			});
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

impl<DataType: PipelineData, ContextType: PipelineJobContext<DataType>> Node<DataType>
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
		_target_port: PipelinePortID,
		_input_data: DataType,
	) -> Result<(), RunNodeError> {
		unreachable!("Input nodes do not have any input ports.")
	}

	fn run(
		&mut self,
		send_data: &dyn Fn(PipelinePortID, DataType) -> Result<(), RunNodeError>,
	) -> Result<NodeState, RunNodeError> {
		send_data(PipelinePortID::new("out"), self.value.clone())?;
		Ok(NodeState::Done)
	}
}
