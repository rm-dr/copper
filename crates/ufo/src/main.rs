use anyhow::Result;
use std::{
	path::{Path, PathBuf},
	sync::{Arc, Mutex},
	thread,
	time::Duration,
};
use walkdir::WalkDir;

use ufo_metadb::{
	api::{AttributeOptions, MetaDb},
	data::{HashType, MetaDbDataStub},
	sqlite::db::SQLiteMetaDB,
};
use ufo_pipeline::{
	api::PipelineNodeState,
	runner::runner::{PipelineRunConfig, PipelineRunner},
};
use ufo_pipeline_nodes::{data::UFOData, nodetype::UFONodeType, UFOContext};

//mod log;

fn main() -> Result<()> {
	tracing_subscriber::fmt()
		//.with_env_filter("ufo_pipeline=debug")
		.with_env_filter("ufo_pipeline=error")
		.without_time()
		.with_ansi(true)
		//.with_max_level(Level::DEBUG)
		//.event_format(log::LogFormatter::new(true))
		.init();

	let d = PathBuf::from("./db");
	std::fs::create_dir(&d).unwrap();

	let database = {
		let mut d = SQLiteMetaDB::connect(&d).unwrap();

		let x = d.add_class("AudioFile").unwrap();
		let cover_art = d.add_class("CoverArt").unwrap();

		d.add_attr(x, "title", MetaDbDataStub::Text, AttributeOptions::new())
			.unwrap();
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
		database: Arc::new(Mutex::new(database)),
		blob_channel_capacity: 10,
		blob_fragment_size: 1_000,
	};

	// Prep runner
	let mut runner: PipelineRunner<UFONodeType> = PipelineRunner::new(
		PipelineRunConfig {
			node_threads: 1,
			max_active_jobs: 1,
		},
		ctx.clone(),
	);
	runner.add_pipeline(Path::new("pipelines/cover.toml"), "cover".into())?;
	runner.add_pipeline(Path::new("pipelines/audiofile.toml"), "audio".into())?;

	for entry in WalkDir::new("./data") {
		let entry = entry.unwrap();
		if entry.path().is_file() {
			runner.add_job(
				&"audio".into(),
				vec![UFOData::Path(Arc::new(entry.path().into()))],
			);
		}
	}

	loop {
		//thread::sleep(Duration::from_secs(1));
		runner.run()?;

		let mut has_active_job = false;
		for p in runner.iter_active_jobs() {
			has_active_job = true;
			/*
			for l in p.get_pipeline().iter_node_labels() {
				println!(
					"{} {l}",
					match p.get_node_status(l).unwrap() {
						(true, _) => "r",
						(false, PipelineNodeState::Done) => "D",
						(false, PipelineNodeState::Pending(_)) => "p",
					}
				);
			}
			*/
		}
		//println!("\n");

		while let Some(x) = runner.pop_completed_job() {
			println!("{x:?}");
		}

		if !has_active_job {
			return Ok(());
		}
	}
}
