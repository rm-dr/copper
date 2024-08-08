use std::{
	fs::File,
	io::{self, Read},
	path::PathBuf,
	sync::Arc,
};
use ufo_util::data::{AudioFormat, BinaryFormat, PipelineData};

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

	fn run(self) -> Result<Vec<Arc<PipelineData>>, Self::ErrorKind> {
		let mut f = File::open(&self.path)?;
		let mut data = Vec::new();
		f.read_to_end(&mut data)?;

		let file_format = match self.path.extension().unwrap().to_str().unwrap() {
			"flac" => BinaryFormat::Audio(AudioFormat::Flac),
			"mp3" => BinaryFormat::Audio(AudioFormat::Mp3),
			_ => BinaryFormat::Blob,
		};

		return Ok(vec![
			// Path
			Arc::new(PipelineData::Text(self.path.to_str().unwrap().to_string())),
			// Data
			Arc::new(PipelineData::Binary {
				format: file_format,
				data,
			}),
		]);
	}
}
