use std::{
	collections::HashMap,
	fs::File,
	io::{self, Read},
	path::PathBuf,
	sync::Arc,
};

use super::Ingest;
use ufo_pipeline::{
	data::{AudioFormat, BinaryFormat, PipelineData},
	syntax::labels::PipelinePortLabel,
};

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

	fn injest(
		self,
	) -> Result<HashMap<PipelinePortLabel, Option<Arc<PipelineData>>>, Self::ErrorKind> {
		let mut f = File::open(&self.path)?;
		let mut data = Vec::new();
		f.read_to_end(&mut data)?;

		let file_format = match self.path.extension().unwrap().to_str().unwrap() {
			"flac" => BinaryFormat::Audio(AudioFormat::Flac),
			"mp3" => BinaryFormat::Audio(AudioFormat::Mp3),
			_ => BinaryFormat::Blob,
		};

		return Ok(HashMap::from([
			(
				"path".into(),
				Some(Arc::new(PipelineData::Text(
					self.path.to_str().unwrap().to_string(),
				))),
			),
			(
				"data".into(),
				Some(Arc::new(PipelineData::Binary {
					format: file_format,
					data,
				})),
			),
		]));
	}
}
