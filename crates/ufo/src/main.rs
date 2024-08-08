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
use ufo_pipeline::runner::runner::{PipelineRunConfig, PipelineRunner};
use ufo_pipeline_nodes::{nodetype::UFONodeType, UFOContext};

//mod log;

fn main() -> Result<()> {
	tracing_subscriber::fmt()
		.with_env_filter("ufo_pipeline=debug")
		.without_time()
		.with_ansi(false)
		//.with_max_level(Level::DEBUG)
		//.event_format(log::LogFormatter::new(true))
		.init();

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
		blob_channel_capacity: 10,
		blob_fragment_size: 1_000_000,
	};

	// Prep runner
	let mut runner: PipelineRunner<UFONodeType> =
		PipelineRunner::new(PipelineRunConfig { node_threads: 1 }, ctx.clone());
	runner.add_pipeline(Path::new("pipelines/cover.toml"), "cover".into())?;
	runner.add_pipeline(Path::new("pipelines/audiofile.toml"), "audio".into())?;

	for p in ["data/freeze.flac"] {
		runner.run(&"audio".into(), vec![MetaDbData::Path(Arc::new(p.into()))])?;
	}

	Ok(())
}
