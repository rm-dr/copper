use futures::executor::block_on;
use smartstring::{LazyCompact, SmartString};
use std::io;
use ufo_storage::api::Dataset;
use ufo_util::data::{PipelineData, PipelineDataType};

use super::PipelineOutput;

pub struct StorageOutput<'a, DatasetType>
where
	DatasetType: Dataset,
{
	dataset: &'a mut DatasetType,
	class: DatasetType::ClassHandle,
	attrs: Vec<(SmartString<LazyCompact>, PipelineDataType)>,
}

impl<'a, DatasetType: Dataset> StorageOutput<'a, DatasetType> {
	pub fn new(
		dataset: &'a mut DatasetType,
		class: DatasetType::ClassHandle,
		attrs: Vec<(SmartString<LazyCompact>, PipelineDataType)>,
	) -> Self {
		StorageOutput {
			dataset,
			class,
			attrs,
		}
	}
}

impl<'a, DatasetType: Dataset> PipelineOutput for StorageOutput<'a, DatasetType> {
	type ErrorKind = io::Error;

	fn run(&mut self, data: Vec<&PipelineData>) -> Result<(), Self::ErrorKind> {
		// TODO: better enforce arg type / arg number
		assert!(data.len() == self.attrs.len());

		// TODO: errors
		let i = block_on(self.dataset.add_item(self.class)).unwrap();

		// TODO: partial add
		// TODO: make sure attrs exist
		for ((attr_name, _), data) in self.attrs.iter().zip(data.iter()) {
			let a = block_on(self.dataset.get_attr(attr_name)).unwrap().unwrap();
			block_on(self.dataset.item_set_attr(i, a, data)).unwrap();
		}

		Ok(())
	}
}
