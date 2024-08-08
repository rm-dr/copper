use anyhow::Result;
use std::{
	path::Path,
	sync::{Arc, Mutex},
};
use ufo_pipeline::{data::PipelineData, runner::runner::PipelineRunner};
use ufo_pipeline_nodes::{nodetype::UFONodeType, UFOContext};
use ufo_storage::{
	api::{AttributeOptions, Dataset},
	sea::dataset::SeaDataset,
	StorageDataType,
};

fn main() -> Result<()> {
	// Make dataset
	let dataset = {
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

	let ctx = Arc::new(UFOContext {
		dataset: Mutex::new(dataset),
	});

	// Prep runner
	let mut runner: PipelineRunner<UFONodeType> = PipelineRunner::new(4);
	runner.add_pipeline(
		ctx.clone(),
		Path::new("pipelines/cover.toml"),
		"cover".into(),
	)?;
	runner.add_pipeline(
		ctx.clone(),
		Path::new("pipelines/audiofile.toml"),
		"audio".into(),
	)?;

	for p in ["data/freeze.flac"] {
		runner.run(
			ctx.clone(),
			"audio".into(),
			vec![PipelineData::Text(Arc::new(p.into()))],
		)?;
	}

	Ok(())
}
