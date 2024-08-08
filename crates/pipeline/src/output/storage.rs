use std::io;
use ufo_storage::api::Dataset;
use ufo_util::data::PipelineData;

use super::PipelineOutput;

pub struct StorageOutput<'a, DatasetType>
where
	DatasetType: Dataset,
{
	dataset: &'a mut DatasetType,
	class: DatasetType::ClassHandle,
}

impl<'a, DatasetType: Dataset> StorageOutput<'a, DatasetType> {
	pub fn new(dataset: &'a mut DatasetType, class: DatasetType::ClassHandle) -> Self {
		StorageOutput { dataset, class }
	}
}

impl<'a, DatasetType: Dataset> PipelineOutput for StorageOutput<'a, DatasetType> {
	type ErrorKind = io::Error;

	fn export(&mut self, data: Vec<Option<&PipelineData>>) -> Result<(), Self::ErrorKind> {
		assert!(data.len() == self.dataset.class_num_attrs(self.class));

		self.dataset
			.add_item_with_attrs(self.class, &data[..])
			.unwrap();

		Ok(())
	}
}
