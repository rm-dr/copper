use anyhow::Result;
use std::path::Path;
use ufo_pipeline::{
	input::{file::FileInput, PipelineInput, PipelineInputKind},
	runner::PipelineRunner,
};
use ufo_storage::{
	api::{AttributeOptions, Dataset},
	sea::dataset::SeaDataset,
	StorageDataType,
};

fn main() -> Result<()> {
	// Make dataset
	let mut dataset = {
		let mut d = SeaDataset::new("sqlite:./test.sqlite?mode=rwc", "ufo_db");
		d.connect()?;
		let x = d.add_class("AudioFile").unwrap();
		d.add_attr(x, "album", StorageDataType::Text, AttributeOptions::new())
			.unwrap();
		d.add_attr(x, "artist", StorageDataType::Text, AttributeOptions::new())
			.unwrap();
		d.add_attr(
			x,
			"albumartist",
			StorageDataType::Text,
			AttributeOptions::new(),
		)
		.unwrap();
		d.add_attr(
			x,
			"tracknumber",
			StorageDataType::Text,
			AttributeOptions::new(),
		)
		.unwrap();
		d.add_attr(x, "year", StorageDataType::Text, AttributeOptions::new())
			.unwrap();
		d.add_attr(x, "genre", StorageDataType::Text, AttributeOptions::new())
			.unwrap();
		d.add_attr(x, "ISRC", StorageDataType::Text, AttributeOptions::new())
			.unwrap();
		d.add_attr(x, "lyrics", StorageDataType::Text, AttributeOptions::new())
			.unwrap();

		d.add_attr(
			x,
			"audio_data",
			StorageDataType::Binary,
			AttributeOptions::new(),
		)
		.unwrap();

		let x = d.add_class("CoverArt").unwrap();
		d.add_attr(
			x,
			"image_data",
			StorageDataType::Binary,
			AttributeOptions::new(),
		)
		.unwrap();
		d.add_attr(
			x,
			"content_hash",
			StorageDataType::Text,
			AttributeOptions::new().unique(true),
		)
		.unwrap();
		d
	};

	// Prep runner
	let mut runner = PipelineRunner::new(&mut dataset, 4);
	runner.add_pipeline(Path::new("pipelines/cover.toml"), "cover".into())?;
	runner.add_pipeline(Path::new("pipelines/audiofile.toml"), "audio".into())?;

	for p in ["data/freeze.flac"] {
		let input = match &runner
			.get_pipeline("audio".into())
			.unwrap()
			.get_config()
			.input
		{
			PipelineInputKind::File => {
				let f = FileInput::new(p.into());
				f.run().unwrap()
			}
			PipelineInputKind::Plain { .. } => unreachable!(),
		};

		runner.run("audio".into(), input)?;
	}

	Ok(())
}
