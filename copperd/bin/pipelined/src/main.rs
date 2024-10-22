use aws_config::{BehaviorVersion, Region};
use aws_sdk_s3::config::Credentials;
use config::{PipelinedConfig, ASYNC_POLL_AWAIT_MS};
use copper_jobqueue::{
	base::{
		client::JobQueueClient,
		errors::{BuildErrorJobError, FailJobError, GetQueuedJobError, SuccessJobError},
	},
	postgres::{PgJobQueueClient, PgJobQueueOpenError},
};
use copper_pipelined::{
	data::{BytesSource, PipeData},
	CopperContext, JobRunResult,
};
use copper_storaged::{client::ReqwestStoragedClient, AttrData, Transaction};
use copper_util::{load_env, s3client::S3Client, LoadedEnv};
use pipeline::runner::{DoneJobState, PipelineRunner, StartJobError};
use std::{collections::BTreeMap, sync::Arc};
use tokio::sync::Mutex;
use tracing::{error, info, trace};

mod config;
mod pipeline;

// #[tokio::main(flavor = "multi_thread", worker_threads = 10)]
// #[tokio::main(flavor = "current_thread")]
#[tokio::main]
async fn main() {
	let config_res = match load_env::<PipelinedConfig>() {
		Ok(x) => x,
		Err(err) => {
			println!("Error while loading .env: {err}");
			std::process::exit(1);
		}
	};

	let config: Arc<PipelinedConfig> = Arc::new(config_res.get_config().clone());

	tracing_subscriber::fmt()
		.with_env_filter(config.pipelined_loglevel.get_config())
		.without_time()
		.with_ansi(true)
		.init();

	// Do this now, logging wasn't available earlier
	match config_res {
		LoadedEnv::FoundFile { config, path } => {
			info!(message = "Loaded config from .env", ?path, ?config);
		}
		LoadedEnv::OnlyVars(config) => {
			info!(
				message = "No `.env` found, loaded config from environment",
				?config
			);
		}
	};

	let cred = Credentials::new(
		&config.pipelined_objectstore_key_id,
		&config.pipelined_objectstore_key_secret,
		None,
		None,
		"pipelined .env",
	);

	// Config for minio
	let s3_config = aws_sdk_s3::config::Builder::new()
		.behavior_version(BehaviorVersion::v2024_03_28())
		.endpoint_url(&config.pipelined_objectstore_url)
		.credentials_provider(cred)
		.region(Region::new("us-west"))
		.force_path_style(true)
		.build();

	let client = Arc::new(S3Client::new(aws_sdk_s3::Client::from_conf(s3_config)).await);

	// Create blobstore bucket if it doesn't exist
	match client
		.create_bucket(&config.pipelined_objectstore_bucket)
		.await
	{
		Ok(false) => {}
		Ok(true) => {
			info!(
				message = "Created storage bucket because it didn't exist",
				bucket = config.pipelined_objectstore_bucket
			);
		}
		Err(error) => {
			error!(
				message = "Error while creating storage bucket",
				bucket = config.pipelined_objectstore_bucket,
				?error
			);
		}
	}

	trace!(message = "Initializing job queue client");
	let jobqueue_client = match PgJobQueueClient::open(&config.pipelined_jobqueue_db).await {
		Ok(db) => Arc::new(db),
		Err(PgJobQueueOpenError::Database(e)) => {
			error!(message = "SQL error while opening job queue database", err = ?e);
			std::process::exit(1);
		}
		Err(PgJobQueueOpenError::Migrate(e)) => {
			error!(message = "Migration error while opening job queue database", err = ?e);
			std::process::exit(1);
		}
	};
	trace!(message = "Successfully initialized job queue client");

	//
	// MARK: Prep runner
	//
	let mut runner: PipelineRunner<JobRunResult, PipeData, CopperContext> = PipelineRunner::new();

	{
		// Base nodes
		use pipelined_basic::register;
		match register(runner.mut_dispatcher()) {
			Ok(()) => {}
			Err(error) => {
				error!(
					message = "Could not register nodes",
					module = "basic",
					?error
				);
				std::process::exit(1);
			}
		};
	}

	{
		// Storaged nodes
		use pipelined_storaged::register;
		match register(runner.mut_dispatcher()) {
			Ok(()) => {}
			Err(error) => {
				error!(
					message = "Could not register nodes",
					module = "storaged",
					?error
				);
				std::process::exit(1);
			}
		};
	}

	{
		// Audiofile nodes
		use pipelined_audiofile::nodes::register;
		match register(runner.mut_dispatcher()) {
			Ok(()) => {}
			Err(error) => {
				error!(
					message = "Could not register nodes",
					module = "audiofile",
					?error
				);
				std::process::exit(1);
			}
		};
	}

	trace!(message = "Initializing storaged client");
	let storaged_client = match ReqwestStoragedClient::new(
		config.pipelined_storaged_addr.clone(),
		&config.pipelined_storaged_secret,
	) {
		Ok(x) => Arc::new(x),
		Err(error) => {
			error!(message = "Could not initialize storaged client", ?error);
			std::process::exit(1);
		}
	};
	trace!(message = "Successfully initialized storaged client");

	loop {
		match runner.check_done_jobs().await {
			Ok(None) => {}

			Ok(Some(DoneJobState::Success { job_id, result })) => {
				info!(
					message = "Job finished successfully",
					job_id = ?job_id
				);

				match jobqueue_client.success_job(&job_id, result).await {
					Ok(()) => {}

					Err(SuccessJobError::DbError(error)) => {
						error!(
							message = "DB error while marking job `Success`",
							?job_id,
							?error
						);
					}

					Err(SuccessJobError::NotRunning) => {
						error!(
							message = "Tried to mark a job that isn't running as `Success`",
							?job_id
						);
					}
				};
			}

			Ok(Some(DoneJobState::Failed { job_id, error })) => {
				info!(message = "Job failed", ?job_id, ?error);

				match jobqueue_client.fail_job(&job_id).await {
					Ok(()) => {}

					Err(FailJobError::DbError(error)) => {
						error!(
							message = "DB error while marking job `Failed`",
							?job_id,
							?error
						);
					}

					Err(FailJobError::NotRunning) => {
						error!(
							message = "Tried to mark a job that isn't running as `Failed`",
							?job_id
						);
					}
				}
			}

			Err(error) => {
				error!(message = "Join error!", ?error);
				panic!("Join error! {error:?}");
			}
		}

		if runner.n_running_jobs() < config.pipelined_max_running_jobs {
			// Run the oldest job off the queue
			let next = match jobqueue_client.get_queued_job().await {
				Ok(x) => x,
				Err(GetQueuedJobError::DbError(error)) => {
					error!(message = "DB error while getting job", ?error);
					tokio::time::sleep(std::time::Duration::from_secs(5)).await;
					continue;
				}
			};

			if let Some(job) = next {
				let mut input = BTreeMap::new();
				for (name, value) in job.input {
					match value {
						AttrData::Blob { bucket, key } => input.insert(
							name,
							PipeData::Blob {
								source: BytesSource::S3 { bucket, key },
							},
						),

						// This should never fail, we handle all special cases above
						_ => input.insert(name, PipeData::try_from(value).unwrap()),
					};
				}

				let res = runner.start_job(
					CopperContext {
						blob_fragment_size: config.pipelined_blob_fragment_size,
						stream_channel_capacity: config.pipelined_stream_channel_size,
						job_id: job.job_id.as_str().into(),
						run_by_user: job.owned_by.clone(),
						storaged_client: storaged_client.clone(),
						objectstore_blob_bucket: config
							.pipelined_objectstore_bucket
							.as_str()
							.into(),
						objectstore_client: client.clone(),
						transaction: Mutex::new(Transaction::new()),
					},
					job.pipeline,
					&job.job_id,
					input,
				);

				match res {
					Ok(()) => {}
					Err(StartJobError::BuildError(err)) => {
						match jobqueue_client
							.builderror_job(&job.job_id, &format!("{:?}", err))
							.await
						{
							Ok(()) => {}

							Err(BuildErrorJobError::DbError(error)) => {
								error!(
									message = "DB error while marking job `BuildError`",
									job_id = ?job.job_id,
									?error
								);
							}

							Err(BuildErrorJobError::NotRunning) => {
								error!(
									message = "Tried to mark a job that isn't running as `BuildError`",
									job_id = ?job.job_id
								);
							}
						}
					}
				}
			}
		}

		// Sleep a little bit so we don't waste cpu cycles.
		tokio::time::sleep(std::time::Duration::from_millis(ASYNC_POLL_AWAIT_MS)).await;
	}
}
