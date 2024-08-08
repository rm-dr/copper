use smartstring::{LazyCompact, SmartString};
use ufo_pipeline::{
	api::{PipelineNode, PipelineNodeState},
	errors::PipelineError,
};
use ufo_storage::{
	api::{AttrHandle, ClassHandle},
	data::{StorageData, StorageDataStub},
};

use crate::UFOContext;

pub struct StorageOutput {
	class: ClassHandle,
	attrs: Vec<(AttrHandle, SmartString<LazyCompact>, StorageDataStub)>,
	data: Vec<StorageData>,
}

impl StorageOutput {
	pub fn new(
		class: ClassHandle,
		attrs: Vec<(AttrHandle, SmartString<LazyCompact>, StorageDataStub)>,
	) -> Self {
		StorageOutput {
			class,
			attrs,
			data: Vec::new(),
		}
	}
}

impl PipelineNode for StorageOutput {
	type NodeContext = UFOContext;
	type DataType = StorageData;

	fn init<F>(
		&mut self,
		_ctx: &Self::NodeContext,
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
		ctx: &Self::NodeContext,
		send_data: F,
	) -> Result<PipelineNodeState, PipelineError>
	where
		F: Fn(usize, Self::DataType) -> Result<(), PipelineError>,
	{
		let mut d = ctx.dataset.lock().unwrap();

		let mut attrs = Vec::new();
		for ((attr, _, _), data) in self.attrs.iter().zip(self.data.iter()) {
			attrs.push((*attr, data.clone()));
		}

		let item = d.add_item(self.class, &attrs).unwrap();

		send_data(
			0,
			StorageData::Reference {
				class: self.class,
				item,
			},
		)?;

		Ok(PipelineNodeState::Done)
	}
}
