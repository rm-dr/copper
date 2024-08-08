use anyhow::Result;
use std::{
	path::Path,
	sync::{Arc, Mutex},
};
use ufo_pipeline::runner::runner::PipelineRunner;
use ufo_pipeline_nodes::{data::UFOData, nodetype::UFONodeType, UFOContext};
use ufo_storage::{
	api::{AttributeOptions, Dataset},
	data::StorageDataType,
	sqlite::dataset::SQLiteDataset,
};

fn main() -> Result<()> {
	// Make dataset
	let dataset = {
		let mut d = SQLiteDataset::new("sqlite:./test.sqlite?mode=rwc");
		d.connect().unwrap();

		let x = d.add_class("AudioFile").unwrap();
		let cover_art = d.add_class("CoverArt").unwrap();

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
			"cover_art",
			StorageDataType::Reference { class: cover_art },
			AttributeOptions::new(),
		)
		.unwrap();

		d.add_attr(
			x,
			"audio_data",
			StorageDataType::Binary,
			AttributeOptions::new(),
		)
		.unwrap();

		d.add_attr(
			cover_art,
			"image_data",
			StorageDataType::Binary,
			AttributeOptions::new(),
		)
		.unwrap();
		d.add_attr(
			cover_art,
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
			vec![UFOData::Path(Arc::new(p.into()))],
		)?;
	}

	Ok(())
}
