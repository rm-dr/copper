use anyhow::Result;
use std::{
	path::Path,
	sync::{Arc, Mutex},
};
use ufo_pipeline::runner::runner::PipelineRunner;
use ufo_pipeline_nodes::{nodetype::UFONodeType, UFOContext};
use ufo_storage::{
	api::{AttributeOptions, Dataset},
	data::{HashType, StorageData, StorageDataStub},
	sqlite::dataset::SQLiteDataset,
};

fn main() -> Result<()> {
	// Make dataset
	let dataset = {
		let mut d = SQLiteDataset::new("sqlite:./test.sqlite?mode=rwc");
		d.connect().unwrap();

		let x = d.add_class("AudioFile").unwrap();
		let cover_art = d.add_class("CoverArt").unwrap();

		d.add_attr(x, "album", StorageDataStub::Text, AttributeOptions::new())
			.unwrap();
		d.add_attr(x, "artist", StorageDataStub::Text, AttributeOptions::new())
			.unwrap();
		d.add_attr(
			x,
			"albumartist",
			StorageDataStub::Text,
			AttributeOptions::new(),
		)
		.unwrap();
		d.add_attr(
			x,
			"tracknumber",
			StorageDataStub::Text,
			AttributeOptions::new(),
		)
		.unwrap();
		d.add_attr(x, "year", StorageDataStub::Text, AttributeOptions::new())
			.unwrap();
		d.add_attr(x, "genre", StorageDataStub::Text, AttributeOptions::new())
			.unwrap();
		d.add_attr(x, "ISRC", StorageDataStub::Text, AttributeOptions::new())
			.unwrap();
		d.add_attr(x, "lyrics", StorageDataStub::Text, AttributeOptions::new())
			.unwrap();
		d.add_attr(
			x,
			"cover_art",
			StorageDataStub::Reference { class: cover_art },
			AttributeOptions::new(),
		)
		.unwrap();

		d.add_attr(
			x,
			"audio_data",
			StorageDataStub::Binary,
			AttributeOptions::new(),
		)
		.unwrap();

		d.add_attr(
			cover_art,
			"image_data",
			StorageDataStub::Binary,
			AttributeOptions::new(),
		)
		.unwrap();
		d.add_attr(
			cover_art,
			"content_hash",
			StorageDataStub::Hash {
				hash_type: HashType::SHA256,
			},
			AttributeOptions::new().unique(true),
		)
		.unwrap();
		d
	};

	let ctx = UFOContext {
		dataset: Arc::new(Mutex::new(dataset)),
	};

	// Prep runner
	let mut runner: PipelineRunner<UFONodeType> = PipelineRunner::new(ctx.clone(), 4);
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
		runner.run(&"audio".into(), vec![StorageData::Path(Arc::new(p.into()))])?;
	}

	Ok(())
}
