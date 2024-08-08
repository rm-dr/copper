use anyhow::Result;
use clap::{Parser, Subcommand};
use indicatif::ProgressBar;
use std::{
	ffi::OsStr,
	path::PathBuf,
	sync::{Arc, Mutex},
	thread,
	time::Duration,
};
use ufo_blobstore::fs::store::FsBlobStore;
use walkdir::WalkDir;

use ufo_metadb::{
	api::{AttributeOptions, MetaDb, MetaDbNew},
	data::{HashType, MetaDbDataStub},
	sqlite::db::SQLiteMetaDB,
};
use ufo_pipeline::{
	labels::PipelineLabel,
	runner::runner::{PipelineRunConfig, PipelineRunner},
};
use ufo_pipeline_nodes::{data::UFOData, nodetype::UFONodeType, UFOContext};

//mod log;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Args {
	#[command(subcommand)]
	command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
	New {
		target_dir: Option<PathBuf>,
	},
	Import {
		pipeline: String,
		args: Vec<String>,

		#[arg(long, default_value = ".")]
		db_root: PathBuf,
	},
}

fn main() -> Result<()> {
	tracing_subscriber::fmt()
		//.with_env_filter("ufo_pipeline=debug")
		.with_env_filter("ufo_pipeline=error")
		.without_time()
		.with_ansi(true)
		//.with_max_level(Level::DEBUG)
		//.event_format(log::LogFormatter::new(true))
		.init();

	let cli = Args::parse();

	match cli.command {
		Commands::New { target_dir } => {
			let db_root = if let Some(p) = target_dir {
				p
			} else {
				PathBuf::from(".")
			};

			if db_root.is_dir() {
				if db_root.read_dir().unwrap().next().is_some() {
					println!("Target directory isn't empty");
					return Ok(());
				}
			} else if db_root.exists() {
				println!("Target exists and isn't a directory");
				return Ok(());
			} else {
				std::fs::create_dir(&db_root).unwrap();
			}

			SQLiteMetaDB::<FsBlobStore>::create(&db_root).unwrap();

			let pipeline_dir = db_root.join("pipelines");
			std::fs::create_dir(&pipeline_dir).unwrap();

			// Everything below this point should be done in UI
			{
				let mut db = SQLiteMetaDB::<FsBlobStore>::open(&db_root).unwrap();

				let x = db.add_class("AudioFile").unwrap();
				let cover_art = db.add_class("CoverArt").unwrap();

				db.add_attr(x, "title", MetaDbDataStub::Text, AttributeOptions::new())
					.unwrap();
				db.add_attr(x, "album", MetaDbDataStub::Text, AttributeOptions::new())
					.unwrap();
				db.add_attr(x, "artist", MetaDbDataStub::Text, AttributeOptions::new())
					.unwrap();
				db.add_attr(
					x,
					"albumartist",
					MetaDbDataStub::Text,
					AttributeOptions::new(),
				)
				.unwrap();
				db.add_attr(
					x,
					"tracknumber",
					MetaDbDataStub::Text,
					AttributeOptions::new(),
				)
				.unwrap();
				db.add_attr(x, "year", MetaDbDataStub::Text, AttributeOptions::new())
					.unwrap();
				db.add_attr(x, "genre", MetaDbDataStub::Text, AttributeOptions::new())
					.unwrap();
				db.add_attr(x, "ISRC", MetaDbDataStub::Text, AttributeOptions::new())
					.unwrap();
				db.add_attr(x, "lyrics", MetaDbDataStub::Text, AttributeOptions::new())
					.unwrap();
				db.add_attr(
					x,
					"cover_art",
					MetaDbDataStub::Reference { class: cover_art },
					AttributeOptions::new(),
				)
				.unwrap();

				db.add_attr(
					x,
					"audio_data",
					MetaDbDataStub::Blob,
					AttributeOptions::new(),
				)
				.unwrap();

				db.add_attr(
					cover_art,
					"image_data",
					MetaDbDataStub::Binary,
					AttributeOptions::new(),
				)
				.unwrap();
				db.add_attr(
					cover_art,
					"content_hash",
					MetaDbDataStub::Hash {
						hash_type: HashType::SHA256,
					},
					AttributeOptions::new().unique(true),
				)
				.unwrap();
			}
		}

		Commands::Import {
			pipeline,
			args,
			db_root,
		} => {
			let database = SQLiteMetaDB::open(&db_root).unwrap();

			let ctx = UFOContext {
				database: Arc::new(Mutex::new(database)),
				blob_channel_capacity: 10,
				blob_fragment_size: 100_000,
			};

			// Prep runner
			let mut runner: PipelineRunner<UFONodeType> = PipelineRunner::new(
				PipelineRunConfig {
					node_threads: 2,
					max_active_jobs: 1,
				},
				ctx.clone(),
			);

			// TODO: pipeline dir stored in db
			for entry in WalkDir::new(db_root.join("pipelines")) {
				let entry = entry.unwrap();
				if entry.path().is_file() {
					if entry.path().extension() != Some(OsStr::new("toml")) {
						panic!()
					}
					runner.add_pipeline(
						entry.path(),
						entry
							.path()
							.file_name()
							.unwrap()
							.to_str()
							.unwrap()
							.strip_suffix(".toml")
							.unwrap()
							.to_string(),
					)?;
				}
			}

			let pipeline: PipelineLabel = pipeline.into();
			if runner.get_pipeline(&pipeline).is_none() {
				println!("Pipeline not found: {}", pipeline);
				return Ok(());
			}

			let spin = ProgressBar::new_spinner();
			spin.enable_steady_tick(Duration::from_millis(50));
			let mut n_jobs = 0;

			let p = PathBuf::from(args.first().unwrap());
			if p.is_file() {
				runner.add_job(&pipeline, vec![UFOData::Path(Arc::new(p))]);
			} else if p.is_dir() {
				for entry in WalkDir::new(&p) {
					let entry = entry.unwrap();
					if entry.path().is_file() {
						thread::sleep(Duration::from_millis(200));
						runner.add_job(
							&pipeline,
							vec![UFOData::Path(Arc::new(entry.path().into()))],
						);
						n_jobs += 1;
						spin.set_message(format!("Scanning {p:?} ({n_jobs} jobs to run)"))
					}
				}
			}
			spin.finish();

			let bar = ProgressBar::new(n_jobs).with_message("Running jobs...");

			loop {
				//thread::sleep(Duration::from_millis(200));
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
					bar.inc(1);
					if x.error.is_some() {
						bar.println(format!("Pipeline failed: {}; {:?}", x.pipeline, x.input));
					}
				}

				if !has_active_job {
					return Ok(());
				}
			}
		}
	}

	Ok(())
}
