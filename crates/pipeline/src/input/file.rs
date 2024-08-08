use std::{
	fs::File,
	io::{self, Read},
	path::PathBuf,
	sync::Arc,
};

use ufo_util::mime::MimeType;

use crate::data::PipelineData;

use super::PipelineInput;

pub struct FileInput {
	path: PathBuf,
}

impl FileInput {
	pub fn new(path: PathBuf) -> Self {
		FileInput { path }
	}
}

impl PipelineInput for FileInput {
	type ErrorKind = io::Error;

	fn run(self) -> Result<Vec<PipelineData>, Self::ErrorKind> {
		let mut f = File::open(&self.path)?;
		let mut data = Vec::new();
		f.read_to_end(&mut data)?;

		let file_format =
			MimeType::from_extension(self.path.extension().unwrap().to_str().unwrap())
				.unwrap_or(MimeType::Blob);

		return Ok(vec![
			// Path
			PipelineData::Text(Arc::new(self.path.to_str().unwrap().to_string())),
			// Data
			PipelineData::Binary {
				format: file_format,
				data: Arc::new(data),
			},
		]);
	}
}
