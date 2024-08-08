use anyhow::Result;
use clap::{Parser, Subcommand};
use crossterm::style::Stylize;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::{fmt::Write, path::PathBuf};
use ufod::{AddJobParams, RunnerStatus, RunningNodeState};
use url::Url;
use walkdir::WalkDir;

use ufo_database::{
	blobstore::fs::store::FsBlobStore,
	metadb::{
		api::{AttributeOptions, UFODb, UFODbNew},
		data::{HashType, MetaDbDataStub},
		sqlite::db::SQLiteDB,
	},
};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Args {
	#[command(subcommand)]
	command: Commands,

	#[arg(long, default_value = "http://localhost:3000")]
	host: Url,
}

#[derive(Debug, Subcommand)]
enum Commands {
	New { target_dir: Option<PathBuf> },
	Import { pipeline: String, args: Vec<String> },
	WatchJobs,
}

fn truncate_front(s: &str, len: usize) -> String {
	if s.len() > len + 3 {
		format!("...{}", &s[s.len() - len..])
	} else {
		String::from(s)
	}
}

fn main() -> Result<()> {
	let cli = Args::parse();

	let spin_style = ProgressStyle::with_template("{spinner:.darkgrey} {msg}")
		.unwrap()
		.tick_chars("⠴⠦⠖⠲⠶");

	/*
		let bar_style = ProgressStyle::with_template(&format!(
			"{} {} {} {}",
			"[{pos}/{len} done]",
			"{bar:30}".dark_grey(),
			"{percent}%",
			"({elapsed}/{eta})"
		))
		.unwrap()
		.progress_chars("⣿⣷⣶⣦⣤⣄⣀");
	*/

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

			SQLiteDB::<FsBlobStore>::create(&db_root).unwrap();

			let pipeline_dir = db_root.join("pipelines");
			std::fs::create_dir(&pipeline_dir).unwrap();

			// Everything below this point should be done in UI
			{
				let mut db = SQLiteDB::<FsBlobStore>::open(&db_root).unwrap();

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

		Commands::Import { pipeline, args } => {
			let client = reqwest::blocking::Client::new();
			let scan_spin = ProgressBar::new_spinner().with_style(spin_style.clone());
			let mut n_jobs = 0;

			let p = PathBuf::from(args.first().unwrap());
			if p.is_file() {
				client
					.post(cli.host.join("add_job").unwrap())
					.json(&AddJobParams {
						pipeline: (&pipeline).into(),
						input: p.into(),
					})
					.send()
					.unwrap();
			} else if p.is_dir() {
				for entry in WalkDir::new(&p) {
					let entry = entry.unwrap();
					if entry.path().is_file() {
						scan_spin.tick();
						client
							.post(cli.host.join("add_job").unwrap())
							.json(&AddJobParams {
								pipeline: (&pipeline).into(),
								input: entry.path().into(),
							})
							.send()
							.unwrap();
						n_jobs += 1;
						scan_spin.set_message(format!(
							"{} {} {}",
							"Scanning".cyan(),
							p.canonicalize().unwrap().to_str().unwrap().dark_grey(),
							format!("(added {n_jobs} jobs)")
						))
					}
				}
			}
			scan_spin.finish();
		}

		Commands::WatchJobs => {
			let client = reqwest::blocking::Client::new();

			let mut active_job_spinners: Vec<(u128, ProgressBar)> = Vec::new();
			//let bar = ProgressBar::new(0).with_style(bar_style.clone());
			let multi_bar = MultiProgress::new();

			let mut is_empty = true;
			let empty_spinner = ProgressBar::new_spinner()
				.with_style(spin_style.clone())
				.with_message(format!(
					"No jobs in queue at {}",
					format!("{}", cli.host).dark_grey().italic()
				));

			multi_bar.insert_from_back(0, empty_spinner.clone());

			loop {
				std::thread::sleep(std::time::Duration::from_millis(100));

				let resp = client.get(cli.host.join("status").unwrap()).send().unwrap();
				let resp: RunnerStatus = serde_json::from_str(&resp.text().unwrap()).unwrap();

				if !resp.running_jobs.is_empty() {
					multi_bar.remove(&empty_spinner);
					is_empty = false;
				} else if is_empty {
					empty_spinner.tick();
					empty_spinner.set_message(format!(
						"No jobs in queue at {} ({} completed)",
						format!("{}", cli.host).dark_grey().italic(),
						resp.finished_jobs
					));
				} else {
					is_empty = true;
					multi_bar.insert_from_back(0, empty_spinner.clone());
				}

				let mut i = 0;
				while i < active_job_spinners.len() {
					let (job_id, spin) = &active_job_spinners[i];
					if resp.running_jobs.iter().all(|x| x.job_id != *job_id) {
						spin.finish_and_clear();
						multi_bar.remove(&spin);
						active_job_spinners.swap_remove(i);
					} else {
						i += 1
					}
				}

				for j in &resp.running_jobs {
					if active_job_spinners.iter().all(|(i, _)| *i != j.job_id) {
						let spin = multi_bar.insert_from_back(
							0,
							ProgressBar::new_spinner().with_style(spin_style.clone()),
						);
						active_job_spinners.push((j.job_id, spin));
					}

					let mut s = String::new();
					for n in &j.node_status {
						s.write_str(&format!(
							"{}",
							match n.state {
								RunningNodeState::Running => "R".yellow(),
								RunningNodeState::Done => "D".dark_green(),
								RunningNodeState::Pending(_) => "#".dark_grey(),
							}
						))
						.unwrap();
					}

					let i = &active_job_spinners
						.iter()
						.find(|(i, _)| *i == j.job_id)
						.unwrap()
						.1;
					i.set_message(format!(
						"{} {} {} {s}   {}",
						"Running".green(),
						j.pipeline,
						format!("[{:>3}]:", j.job_id).dark_grey(),
						format!(
							"Input: {}",
							// TODO: pick one input, depending on type of pipeline
							truncate_front(&j.input_exemplar, 30)
						)
						.dark_grey()
						.italic()
					));
					i.tick();
					//bar.tick();
				}

				/*
				let resp: Vec<CompletedJobStatus> =
					serde_json::from_str(&resp.text().unwrap()).unwrap();
				//multi_bar.println(format!("{:?}", resp));

				for x in &resp {
					if x.error.is_some() {
						multi_bar
							.println(format!(
								"Pipeline failed: {}; {:?}",
								x.pipeline, x.input_exemplar
							))
							.unwrap();
					}

					let i = active_job_spinners
						.iter()
						.enumerate()
						.find(|(_, (i, _))| *i == x.job_id)
						.map(|(i, _)| i);
					if let Some(i) = i {
						//bar.inc(1);
						let x = active_job_spinners.swap_remove(i);
						multi_bar.remove(&x.1);
					}
				}
				*/
			}
		}
	}

	Ok(())
}
