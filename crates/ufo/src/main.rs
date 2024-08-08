use anyhow::Result;
use clap::{Parser, Subcommand};
use crossterm::style::Stylize;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::{
	ffi::OsStr,
	fmt::Write,
	path::PathBuf,
	sync::{Arc, Mutex},
};
use ufo_blobstore::fs::store::FsBlobStore;
use walkdir::WalkDir;

use ufo_metadb::{
	api::{AttributeOptions, MetaDb, MetaDbNew},
	data::{HashType, MetaDbDataStub},
	sqlite::db::SQLiteMetaDB,
};
use ufo_pipeline::{
	api::PipelineNodeState,
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

fn truncate_front(s: &str, len: usize) -> String {
	if s.len() > len + 3 {
		format!("...{}", &s[s.len() - len..])
	} else {
		String::from(s)
	}
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
					AttributeOptions::new().not_null(true),
				)
				.unwrap();
				db.add_attr(
					x,
					"audio_hash",
					MetaDbDataStub::Hash {
						hash_type: HashType::SHA256,
					},
					AttributeOptions::new().not_null(true),
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
					node_threads: 1,
					max_active_jobs: 5,
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

			let multi_bar = MultiProgress::new();
			let spin_style = ProgressStyle::with_template("{spinner:.darkgrey} {msg}")
				.unwrap()
				.tick_chars("⠴⠦⠖⠲⠶");
			let bar_style = ProgressStyle::with_template(&format!(
				"{} {} {} {}",
				"[{pos}/{len} done]",
				"{bar:30}".dark_grey(),
				"{percent}%",
				"({elapsed}/{eta})"
			))
			.unwrap()
			.progress_chars("⣿⣷⣶⣦⣤⣄⣀");

			let scan_spin = ProgressBar::new_spinner().with_style(spin_style.clone());
			multi_bar.add(scan_spin.clone());
			let mut n_jobs = 0;

			let p = PathBuf::from(args.first().unwrap());
			if p.is_file() {
				runner.add_job(&pipeline, vec![UFOData::Path(Arc::new(p))]);
			} else if p.is_dir() {
				for entry in WalkDir::new(&p) {
					let entry = entry.unwrap();
					if entry.path().is_file() {
						scan_spin.tick();
						runner.add_job(
							&pipeline,
							vec![UFOData::Path(Arc::new(entry.path().into()))],
						);
						n_jobs += 1;
						scan_spin.set_message(format!(
							"{} {} {}",
							"Scanning".cyan(),
							p.canonicalize().unwrap().to_str().unwrap().dark_grey(),
							format!("({n_jobs} jobs to run)")
						))
					}
				}
			}
			scan_spin.finish();
			//multi_bar.remove(&scan_spin);

			let mut active_job_spinners = Vec::new();
			let bar = ProgressBar::new(n_jobs).with_style(bar_style.clone());
			multi_bar.insert_after(&scan_spin, bar.clone());

			loop {
				//thread::sleep(Duration::from_millis(10));
				runner.run()?;

				let mut has_active_job = false;
				for (id, job) in runner.iter_active_jobs() {
					has_active_job = true;

					if active_job_spinners.iter().all(|(i, _)| i != id) {
						let spin = multi_bar.insert_before(
							&bar,
							ProgressBar::new_spinner().with_style(spin_style.clone()),
						);
						active_job_spinners.push((*id, spin));
					}

					let mut s = String::new();
					for l in job.get_pipeline().iter_node_labels() {
						s.write_str(&format!(
							"{}",
							match job.get_node_status(l).unwrap() {
								(true, _) => "R".yellow(),
								(false, PipelineNodeState::Done) => "D".dark_green(),
								(false, PipelineNodeState::Pending(_)) => "#".dark_grey(),
							}
						))
						.unwrap();
					}

					let i = &active_job_spinners.iter().find(|(i, _)| i == id).unwrap().1;
					i.set_message(format!(
						"{} {} {} {s}   {}",
						"Running".green(),
						job.get_pipeline().get_name(),
						format!("[{id:>3}]:").dark_grey(),
						format!(
							"Input: {}",
							// TODO: pick one input, depending on type of pipeline
							truncate_front(&format!("{:?}", job.get_input().first().unwrap()), 30)
						)
						.dark_grey()
						.italic()
					));
					i.tick();
					bar.tick();
				}

				while let Some(x) = runner.pop_completed_job() {
					bar.inc(1);
					if x.error.is_some() {
						multi_bar
							.println(format!("Pipeline failed: {}; {:?}", x.pipeline, x.input))
							.unwrap();
					}

					let i = active_job_spinners
						.iter()
						.enumerate()
						.find(|(_, (i, _))| *i == x.job_id)
						.map(|(i, _)| i);
					if let Some(i) = i {
						let x = active_job_spinners.swap_remove(i);
						multi_bar.remove(&x.1);
					}
				}

				if !has_active_job {
					bar.finish();
					return Ok(());
				}
			}
		}
	}

	Ok(())
}
