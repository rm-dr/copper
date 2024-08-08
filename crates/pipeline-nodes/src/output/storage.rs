use std::sync::Arc;

use smartstring::{LazyCompact, SmartString};
use ufo_pipeline::{
	api::{PipelineNode, PipelineNodeState},
	errors::PipelineError,
};
use ufo_storage::api::{ClassHandle, Dataset};

use crate::{
	data::{UFOData, UFODataStub},
	UFOContext,
};

pub struct StorageOutput {
	class: ClassHandle,
	attrs: Vec<(SmartString<LazyCompact>, UFODataStub)>,
	data: Vec<UFOData>,
}

impl StorageOutput {
	pub fn new(class: ClassHandle, attrs: Vec<(SmartString<LazyCompact>, UFODataStub)>) -> Self {
		StorageOutput {
			class,
			attrs,
			data: Vec::new(),
		}
	}
}

impl PipelineNode for StorageOutput {
	type NodeContext = UFOContext;
	type DataType = UFOData;

	fn init<F>(
		&mut self,
		_ctx: Arc<Self::NodeContext>,
		input: Vec<Self::DataType>,
		_send_data: F,
	) -> Result<PipelineNodeState, PipelineError>
	where
		F: Fn(usize, Self::DataType) -> Result<(), PipelineError>,
	{
		assert!(input.len() == self.attrs.len());
		self.data = input;
		Ok(PipelineNodeState::Pending)
	}

	fn run<F>(
		&mut self,
		ctx: Arc<Self::NodeContext>,
		send_data: F,
	) -> Result<PipelineNodeState, PipelineError>
	where
		F: Fn(usize, Self::DataType) -> Result<(), PipelineError>,
	{
		let mut d = ctx.dataset.lock().unwrap();

		// TODO: partial add
		// TODO: make sure attrs exist
		let mut attrs = Vec::new();
		for ((attr_name, _), data) in self.attrs.iter().zip(self.data.iter()) {
			let a = d.get_attr(self.class, attr_name).unwrap().unwrap();
			attrs.push((a, data.to_storage_data()));
		}

		let item = d.add_item(self.class, &attrs).unwrap();

		send_data(
			0,
			UFOData::Reference {
				class: self.class,
				item,
			},
		)?;

		Ok(PipelineNodeState::Done)
	}
}
