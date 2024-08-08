use anyhow::Result;
use std::{
	path::{Path, PathBuf},
	sync::{Arc, Mutex},
};
use ufo_metadb::{
	api::{AttributeOptions, MetaDb},
	data::{HashType, MetaDbData, MetaDbDataStub},
	sqlite::db::SQLiteMetaDB,
};
use ufo_pipeline::runner::runner::PipelineRunner;
use ufo_pipeline_nodes::{nodetype::UFONodeType, UFOContext};

fn main() -> Result<()> {
	let d = PathBuf::from("./db");
	std::fs::create_dir(&d).unwrap();

	// Make dataset
	let dataset = {
		let mut d = SQLiteMetaDB::connect(&d).unwrap();

		let x = d.add_class("AudioFile").unwrap();
		let cover_art = d.add_class("CoverArt").unwrap();

		d.add_attr(x, "album", MetaDbDataStub::Text, AttributeOptions::new())
			.unwrap();
		d.add_attr(x, "artist", MetaDbDataStub::Text, AttributeOptions::new())
			.unwrap();
		d.add_attr(
			x,
			"albumartist",
			MetaDbDataStub::Text,
			AttributeOptions::new(),
		)
		.unwrap();
		d.add_attr(
			x,
			"tracknumber",
			MetaDbDataStub::Text,
			AttributeOptions::new(),
		)
		.unwrap();
		d.add_attr(x, "year", MetaDbDataStub::Text, AttributeOptions::new())
			.unwrap();
		d.add_attr(x, "genre", MetaDbDataStub::Text, AttributeOptions::new())
			.unwrap();
		d.add_attr(x, "ISRC", MetaDbDataStub::Text, AttributeOptions::new())
			.unwrap();
		d.add_attr(x, "lyrics", MetaDbDataStub::Text, AttributeOptions::new())
			.unwrap();
		d.add_attr(
			x,
			"cover_art",
			MetaDbDataStub::Reference { class: cover_art },
			AttributeOptions::new(),
		)
		.unwrap();

		d.add_attr(
			x,
			"audio_data",
			MetaDbDataStub::Blob,
			AttributeOptions::new(),
		)
		.unwrap();

		d.add_attr(
			cover_art,
			"image_data",
			MetaDbDataStub::Binary,
			AttributeOptions::new(),
		)
		.unwrap();
		d.add_attr(
			cover_art,
			"content_hash",
			MetaDbDataStub::Hash {
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
		runner.run(&"audio".into(), vec![MetaDbData::Path(Arc::new(p.into()))])?;
	}

	Ok(())
}
