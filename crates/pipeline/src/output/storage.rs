use smartstring::{LazyCompact, SmartString};
use std::io;
use ufo_storage::api::{ClassHandle, Dataset};

use crate::data::{PipelineData, PipelineDataType};

use super::PipelineOutput;

pub struct StorageOutput<'a> {
	dataset: &'a mut dyn Dataset,
	class: ClassHandle,
	attrs: Vec<(SmartString<LazyCompact>, PipelineDataType)>,
}

impl<'a> StorageOutput<'a> {
	pub fn new(
		dataset: &'a mut dyn Dataset,
		class: ClassHandle,
		attrs: Vec<(SmartString<LazyCompact>, PipelineDataType)>,
	) -> Self {
		StorageOutput {
			dataset,
			class,
			attrs,
		}
	}
}

impl<'a> PipelineOutput for StorageOutput<'a> {
	type ErrorKind = io::Error;

	fn run(&mut self, data: Vec<&PipelineData>) -> Result<(), Self::ErrorKind> {
		// TODO: better enforce arg type / arg number
		assert!(data.len() == self.attrs.len());

		// TODO: errors
		let i = self.dataset.add_item(self.class).unwrap();

		// TODO: partial add
		// TODO: make sure attrs exist
		for ((attr_name, _), data) in self.attrs.iter().zip(data.iter()) {
			let a = self.dataset.get_attr(attr_name).unwrap().unwrap();
			self.dataset
				.item_set_attr(i, a, &data.to_storage_data())
				.unwrap();
		}

		Ok(())
	}
}
