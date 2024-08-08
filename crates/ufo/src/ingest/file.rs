use std::{
	fs::File,
	io::{self, Read},
	path::PathBuf,
	sync::Arc,
};

use super::Ingest;
use ufo_pipeline::data::{AudioFormat, BinaryFormat, PipelineData};

pub struct FileInjest {
	path: PathBuf,
}

impl FileInjest {
	pub fn new(path: PathBuf) -> Self {
		FileInjest { path }
	}
}

impl Ingest for FileInjest {
	type ErrorKind = io::Error;

	fn injest(self) -> Result<Vec<Option<Arc<PipelineData>>>, Self::ErrorKind> {
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
			Some(Arc::new(PipelineData::Text(
				self.path.to_str().unwrap().to_string(),
			))),
			// Data
			Some(Arc::new(PipelineData::Binary {
				format: file_format,
				data,
			})),
		]);
	}
}
