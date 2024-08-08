use anyhow::Result;
use futures::executor::block_on;
use std::path::Path;
use ufo_pipeline::{
	input::{file::FileInput, PipelineInput, PipelineInputKind},
	runner::PipelineRunner,
};
use ufo_storage::{
	api::{AttributeOptions, Dataset},
	sea::dataset::SeaDataset,
};
use ufo_util::data::PipelineDataType;

fn main() -> Result<()> {
	// Make dataset
	let dataset = {
		let mut d = SeaDataset::new("sqlite:./test.sqlite?mode=rwc", "ufo_db");
		block_on(d.connect())?;
		let x = block_on(d.add_class("AudioFile")).unwrap();
		block_on(d.add_attr(x, "album", PipelineDataType::Text, AttributeOptions::new())).unwrap();
		block_on(d.add_attr(x, "artist", PipelineDataType::Text, AttributeOptions::new())).unwrap();
		block_on(d.add_attr(
			x,
			"albumartist",
			PipelineDataType::Text,
			AttributeOptions::new(),
		))
		.unwrap();
		block_on(d.add_attr(
			x,
			"tracknumber",
			PipelineDataType::Text,
			AttributeOptions::new(),
		))
		.unwrap();
		block_on(d.add_attr(x, "year", PipelineDataType::Text, AttributeOptions::new())).unwrap();
		block_on(d.add_attr(x, "genre", PipelineDataType::Text, AttributeOptions::new())).unwrap();
		block_on(d.add_attr(x, "ISRC", PipelineDataType::Text, AttributeOptions::new())).unwrap();
		block_on(d.add_attr(x, "lyrics", PipelineDataType::Text, AttributeOptions::new())).unwrap();

		block_on(d.add_attr(
			x,
			"audio_data",
			PipelineDataType::Binary,
			AttributeOptions::new(),
		))
		.unwrap();

		let x = block_on(d.add_class("CoverArt")).unwrap();
		block_on(d.add_attr(
			x,
			"image_data",
			PipelineDataType::Binary,
			AttributeOptions::new(),
		))
		.unwrap();
		block_on(d.add_attr(
			x,
			"content_hash",
			PipelineDataType::Text,
			AttributeOptions::new().unique(true),
		))
		.unwrap();
		d
	};

	// Prep runner
	let mut runner = PipelineRunner::new(dataset, 4);
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
