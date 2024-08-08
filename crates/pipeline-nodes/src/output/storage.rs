use std::sync::Arc;

use smartstring::{LazyCompact, SmartString};
use ufo_pipeline::{
	data::{PipelineData, PipelineDataType},
	errors::PipelineError,
	node::{PipelineNode, PipelineNodeState},
};
use ufo_storage::api::{ClassHandle, Dataset};

use crate::UFOContext;

pub struct StorageOutput {
	class: ClassHandle,
	attrs: Vec<(SmartString<LazyCompact>, PipelineDataType)>,
	data: Vec<PipelineData>,
}

impl StorageOutput {
	pub fn new(
		class: ClassHandle,
		attrs: Vec<(SmartString<LazyCompact>, PipelineDataType)>,
	) -> Self {
		StorageOutput {
			class,
			attrs,
			data: Vec::new(),
		}
	}
}

impl PipelineNode for StorageOutput {
	type RunContext = UFOContext;

	fn init<F>(
		&mut self,
		_ctx: Arc<Self::RunContext>,
		input: Vec<PipelineData>,
		_send_data: F,
	) -> Result<PipelineNodeState, PipelineError>
	where
		F: Fn(usize, PipelineData) -> Result<(), PipelineError>,
	{
		assert!(input.len() == self.attrs.len());
		self.data = input;
		Ok(PipelineNodeState::Pending)
	}

	fn run<F>(
		&mut self,
		ctx: Arc<Self::RunContext>,
		send_data: F,
	) -> Result<PipelineNodeState, PipelineError>
	where
		F: Fn(usize, PipelineData) -> Result<(), PipelineError>,
	{
		let mut d = ctx.dataset.lock().unwrap();
		let item = d.add_item(self.class).unwrap();

		// TODO: partial add
		// TODO: make sure attrs exist
		for ((attr_name, _), data) in self.attrs.iter().zip(self.data.iter()) {
			let a = d.get_attr(attr_name).unwrap().unwrap();
			d.item_set_attr(item, a, &data.to_storage_data()).unwrap();
		}

		send_data(
			0,
			PipelineData::Reference {
				class: self.class,
				item,
			},
		)?;

		Ok(PipelineNodeState::Done)
	}
}
