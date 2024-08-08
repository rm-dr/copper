use anyhow::Result;
use api::client::UfoApiClient;
use clap::{Parser, Subcommand};
use crossterm::style::Stylize;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::{fmt::Write, fs::File, path::PathBuf};
use ufo_api::{
	data::{ApiData, ApiDataStub},
	pipeline::AddJobParams,
	runner::RunningNodeState,
};
use ufo_database::{api::UFODatabase, database::Database};
use ufo_db_blobstore::fs::store::FsBlobstore;
use ufo_db_metastore::{
	api::AttributeOptions,
	data::{HashType, MetastoreDataStub},
	sqlite::db::SQLiteMetastore,
};
use ufo_db_pipestore::fs::FsPipestore;
use ufo_pipeline::labels::PipelineLabel;
use ufo_util::mime::MimeType;
use url::Url;

mod api;

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
	New {
		target_dir: Option<PathBuf>,
	},
	CreateJob {
		pipeline: PipelineLabel,
		args: Vec<String>,
	},
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

	let api = UfoApiClient::new(cli.host);

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

			Database::<FsBlobstore, SQLiteMetastore, FsPipestore>::create(&db_root).unwrap();

			// Everything below this point should be done in UI
			{
				let database = Database::open(&PathBuf::from("./db")).unwrap();
				let db = database.get_metastore();

				let x = db.add_class("AudioFile").unwrap();
				let cover_art = db.add_class("CoverArt").unwrap();

				db.add_attr(x, "title", MetastoreDataStub::Text, AttributeOptions::new())
					.unwrap();
				db.add_attr(x, "album", MetastoreDataStub::Text, AttributeOptions::new())
					.unwrap();
				db.add_attr(
					x,
					"artist",
					MetastoreDataStub::Text,
					AttributeOptions::new(),
				)
				.unwrap();
				db.add_attr(
					x,
					"albumartist",
					MetastoreDataStub::Text,
					AttributeOptions::new(),
				)
				.unwrap();
				db.add_attr(
					x,
					"tracknumber",
					MetastoreDataStub::Text,
					AttributeOptions::new(),
				)
				.unwrap();
				db.add_attr(x, "year", MetastoreDataStub::Text, AttributeOptions::new())
					.unwrap();
				db.add_attr(x, "genre", MetastoreDataStub::Text, AttributeOptions::new())
					.unwrap();
				db.add_attr(x, "ISRC", MetastoreDataStub::Text, AttributeOptions::new())
					.unwrap();
				db.add_attr(
					x,
					"lyrics",
					MetastoreDataStub::Text,
					AttributeOptions::new(),
				)
				.unwrap();
				db.add_attr(
					x,
					"cover_art",
					MetastoreDataStub::Reference { class: cover_art },
					AttributeOptions::new(),
				)
				.unwrap();

				db.add_attr(
					x,
					"audio_data",
					MetastoreDataStub::Blob,
					AttributeOptions::new().not_null(true),
				)
				.unwrap();
				db.add_attr(
					x,
					"audio_hash",
					MetastoreDataStub::Hash {
						hash_type: HashType::SHA256,
					},
					AttributeOptions::new().not_null(true),
				)
				.unwrap();

				db.add_attr(
					cover_art,
					"image_data",
					MetastoreDataStub::Binary,
					AttributeOptions::new(),
				)
				.unwrap();
				db.add_attr(
					cover_art,
					"content_hash",
					MetastoreDataStub::Hash {
						hash_type: HashType::SHA256,
					},
					AttributeOptions::new().unique(true),
				)
				.unwrap();
			}
		}

		Commands::CreateJob { pipeline, args } => {
			let pipe = if let Some(pipe) = api.get_pipeline(&pipeline) {
				pipe
			} else {
				panic!("bad pipeline");
			};

			let input_node = api.get_pipeline_node(&pipeline, &pipe.input_node).unwrap();

			if input_node.inputs.len() != args.len() {
				panic!("bad arguments")
			}
			let mut input = Vec::new();

			let mut uploadjob = None;

			for (accepted_types, arg) in input_node.inputs.iter().zip(&args) {
				let mut pushed = false;
				for t in accepted_types {
					let parsed_input_data = match t {
						ApiDataStub::Text => Some(ApiData::Text(arg.clone())),
						ApiDataStub::Blob => {
							let path: PathBuf = if let Ok(path) = arg.parse() {
								path
							} else {
								panic!("")
							};

							if !path.is_file() {
								panic!("")
							}

							// Start an upload job if we haven't already
							let info = match &uploadjob {
								Some(x) => x,
								None => {
									uploadjob = Some(api.new_upload_job().unwrap());
									uploadjob.as_ref().unwrap()
								}
							};

							// Open file & detect mime type
							let mut f = File::open(&path).unwrap();
							let mime = MimeType::from_extension(
								&path.extension().unwrap().to_string_lossy().to_string(),
							)
							.unwrap_or(MimeType::Blob);

							// Upload file & get id
							let file_handle = info.upload_file(mime, &mut f).unwrap();
							Some(ApiData::Blob {
								upload_job: info.get_job_id().clone(),
								file_name: file_handle,
							})
						}
						ApiDataStub::Float => arg.parse().ok().map(ApiData::Float),
						ApiDataStub::Integer => arg.parse().ok().map(ApiData::Integer),
						ApiDataStub::PositiveInteger => {
							arg.parse().ok().map(ApiData::PositiveInteger)
						}
						ApiDataStub::Boolean => arg.parse().ok().map(ApiData::Boolean),
					};

					if let Some(p) = parsed_input_data {
						input.push(p);
						pushed = true;
						break;
					}
				}

				if !pushed {
					panic!("failed to add an argument")
				}
			}

			println!("{:?}", input);
			/*
			println!(
				"{:?}",
				api.add_job(AddJobParams {
					pipeline,
					input,
					bound_upload_job: uploadjob.map(|x| x.get_job_id().clone())
				})
			);
			*/
		}

		Commands::WatchJobs => {
			let mut active_job_spinners: Vec<(u128, ProgressBar)> = Vec::new();
			//let bar = ProgressBar::new(0).with_style(bar_style.clone());
			let multi_bar = MultiProgress::new();

			let mut is_empty = true;
			let empty_spinner = ProgressBar::new_spinner()
				.with_style(spin_style.clone())
				.with_message(format!(
					"No jobs in queue at {}",
					format!("{}", api.get_host()).dark_grey().italic()
				));

			multi_bar.insert_from_back(0, empty_spinner.clone());

			loop {
				std::thread::sleep(std::time::Duration::from_millis(100));

				let status = api.get_status();
				if !status.running_jobs.is_empty() {
					multi_bar.remove(&empty_spinner);
					is_empty = false;
				} else if is_empty {
					empty_spinner.tick();
					empty_spinner.set_message(format!(
						"No jobs in queue at {} ({} completed)",
						format!("{}", api.get_host()).dark_grey().italic(),
						status.finished_jobs
					));
				} else {
					is_empty = true;
					multi_bar.insert_from_back(0, empty_spinner.clone());
				}

				let mut i = 0;
				while i < active_job_spinners.len() {
					let (job_id, spin) = &active_job_spinners[i];
					if status.running_jobs.iter().all(|x| x.job_id != *job_id) {
						spin.finish_and_clear();
						multi_bar.remove(&spin);
						active_job_spinners.swap_remove(i);
					} else {
						i += 1
					}
				}

				for j in &status.running_jobs {
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
								RunningNodeState::Pending { .. } => "#".dark_grey(),
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
