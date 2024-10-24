use aws_config::{BehaviorVersion, Region};
use aws_sdk_s3::config::Credentials;
use config::{PiperConfig, ASYNC_POLL_AWAIT_MS};
use copper_itemdb::{
	client::{
		base::{client::ItemdbClient, errors::transaction::ApplyTransactionError},
		postgres::{PgItemdbClient, PgItemdbOpenError},
	},
	transaction::Transaction,
	AttrData,
};
use copper_jobqueue::{
	base::{
		client::JobQueueClient,
		errors::{BuildErrorJobError, FailJobError, GetQueuedJobError, SuccessJobError},
	},
	postgres::{PgJobQueueClient, PgJobQueueOpenError},
};
use copper_piper::{
	data::{BytesSource, PipeData},
	CopperContext, JobRunResult,
};
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
	let config_res = match load_env::<PiperConfig>() {
		Ok(x) => x,
		Err(err) => {
			println!("Error while loading .env: {err}");
			std::process::exit(1);
		}
	};

	let config: Arc<PiperConfig> = Arc::new(config_res.get_config().clone());

	tracing_subscriber::fmt()
		.with_env_filter(config.piper_loglevel.get_config())
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
		&config.piper_objectstore_key_id,
		&config.piper_objectstore_key_secret,
		None,
		None,
		"piper .env",
	);

	// Config for minio
	let s3_config = aws_sdk_s3::config::Builder::new()
		.behavior_version(BehaviorVersion::v2024_03_28())
		.endpoint_url(&config.piper_objectstore_url)
		.credentials_provider(cred)
		.region(Region::new("us-west"))
		.force_path_style(true)
		.build();

	let client = Arc::new(S3Client::new(aws_sdk_s3::Client::from_conf(s3_config)).await);

	// Create blobstore bucket if it doesn't exist
	match client.create_bucket(&config.piper_objectstore_bucket).await {
		Ok(false) => {}
		Ok(true) => {
			info!(
				message = "Created storage bucket because it didn't exist",
				bucket = config.piper_objectstore_bucket
			);
		}
		Err(error) => {
			error!(
				message = "Error while creating storage bucket",
				bucket = config.piper_objectstore_bucket,
				?error
			);
		}
	}

	trace!(message = "Initializing job queue client");
	let jobqueue_client = loop {
		match PgJobQueueClient::open(&config.piper_jobqueue_addr, false).await {
			Ok(db) => break Arc::new(db),
			Err(PgJobQueueOpenError::Database(e)) => {
				error!(message = "SQL error while opening job queue database", err = ?e);
				std::process::exit(1);
			}
			Err(PgJobQueueOpenError::Migrate(e)) => {
				error!(message = "Migration error while opening job queue database", err = ?e);
				std::process::exit(1);
			}
			Err(PgJobQueueOpenError::NotMigrated) => {
				error!(message = "Database not migrated, waiting");
				tokio::time::sleep(std::time::Duration::from_secs(5)).await;
			}
		};
	};
	trace!(message = "Successfully initialized job queue client");

	trace!(message = "Connecting to itemdb");
	// Connect to database
	let itemdb_client = loop {
		match PgItemdbClient::open(&config.piper_itemdb_addr, false).await {
			Ok(db) => break Arc::new(db),
			Err(PgItemdbOpenError::Database(e)) => {
				error!(message = "SQL error while opening item database", err = ?e);
				std::process::exit(1);
			}
			Err(PgItemdbOpenError::Migrate(e)) => {
				error!(message = "Migration error while opening item database", err = ?e);
				std::process::exit(1);
			}
			Err(PgItemdbOpenError::NotMigrated) => {
				error!(message = "Database not migrated, waiting");
				tokio::time::sleep(std::time::Duration::from_secs(5)).await;
			}
		}
	};
	trace!(message = "Successfully connected to itemdb");

	//
	// MARK: Prep runner
	//
	let mut runner: PipelineRunner<JobRunResult, PipeData, CopperContext<PgItemdbClient>> =
		PipelineRunner::new();

	{
		// Base nodes
		use nodes_basic::register;
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
		// Audiofile nodes
		use nodes_audiofile::nodes::register;
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

	loop {
		match runner.check_done_jobs().await {
			Ok(None) => {}

			Ok(Some(DoneJobState::Success { job_id, result })) => {
				info!(
					message = "Job finished successfully, processing transaction",
					job_id = ?job_id
				);

				match itemdb_client.apply_transaction(result.transaction).await {
					Ok(()) => {
						info!(
							message = "Transaction processed successfully",
							job_id = ?job_id
						);

						match jobqueue_client.success_job(&job_id).await {
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

					Err(err) => {
						match &err {
							ApplyTransactionError::DbError(error) => {
								error!(
									message = "DB error while processing transaction",
									?job_id,
									?error
								);
							}

							ApplyTransactionError::AddItemError(error) => {
								info!(
									message = "Failed applying pipeline transaction",
									?job_id,
									?error
								)
							}

							error => {
								info!(
									message = "Failed applying pipeline transaction",
									?job_id,
									?error
								)
							}
						};

						match jobqueue_client
							.fail_job_transaction(&job_id, &format!("{}", err))
							.await
						{
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
						};
					}
				};
			}

			Ok(Some(DoneJobState::Failed { job_id, error })) => {
				info!(message = "Job failed", ?job_id, ?error);

				match jobqueue_client
					.fail_job_run(&job_id, &format!("{}", error))
					.await
				{
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

		if runner.n_running_jobs() < config.piper_max_running_jobs {
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
						blob_fragment_size: config.piper_blob_fragment_size,
						stream_channel_capacity: config.piper_stream_channel_size,
						job_id: job.job_id.as_str().into(),
						run_by_user: job.owned_by,
						itemdb_client: itemdb_client.clone(),
						objectstore_blob_bucket: config.piper_objectstore_bucket.as_str().into(),
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
