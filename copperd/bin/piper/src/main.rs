use aws_config::{BehaviorVersion, Region};
use aws_sdk_s3::config::Credentials;
use config::{PiperConfig, ASYNC_POLL_AWAIT_MS};
use copper_itemdb::{
	client::{ItemdbClient, ItemdbOpenError},
	AttrData,
};
use copper_jobqueue::{
	base::{
		client::JobQueueClient,
		errors::{BuildErrorJobError, FailJobError, GetQueuedJobError, SuccessJobError},
	},
	id::QueuedJobId,
	postgres::{PgJobQueueClient, PgJobQueueOpenError},
};
use copper_piper::{
	base::RunNodeError,
	data::PipeData,
	helpers::{processor::BytesProcessorBuilder, rawbytes::RawBytesSource},
	CopperContext,
};
use copper_util::{load_env, s3client::S3Client, LoadedEnv};
use pipeline::runner::{PipelineRunner, StartJobError};
use sqlx::Acquire;
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

		#[expect(clippy::print_stdout)]
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
	match client
		.create_bucket(&config.piper_objectstore_storage_bucket)
		.await
	{
		Ok(false) => {}
		Ok(true) => {
			info!(
				message = "Created storage bucket because it didn't exist",
				bucket = config.piper_objectstore_storage_bucket
			);
		}
		Err(error) => {
			error!(
				message = "Error while creating storage bucket",
				bucket = config.piper_objectstore_storage_bucket,
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
		match ItemdbClient::open(&config.piper_itemdb_addr, false).await {
			Ok(db) => break Arc::new(db),
			Err(ItemdbOpenError::Database(e)) => {
				error!(message = "SQL error while opening item database", err = ?e);
				std::process::exit(1);
			}
			Err(ItemdbOpenError::Migrate(e)) => {
				error!(message = "Migration error while opening item database", err = ?e);
				std::process::exit(1);
			}
			Err(ItemdbOpenError::NotMigrated) => {
				error!(message = "Database not migrated, waiting");
				tokio::time::sleep(std::time::Duration::from_secs(5)).await;
			}
		}
	};
	trace!(message = "Successfully connected to itemdb");

	//
	// MARK: Prep runner
	//
	let mut runner: PipelineRunner = PipelineRunner::new();

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
		// Run the oldest job off the queue
		let job = match jobqueue_client.get_queued_job().await {
			Ok(x) => x,
			Err(GetQueuedJobError::DbError(error)) => {
				error!(message = "DB error while getting job", ?error);
				tokio::time::sleep(std::time::Duration::from_secs(5)).await;
				continue;
			}
		};

		let job = if let Some(job) = job {
			job
		} else {
			// No job ready, wait a bit...
			tokio::time::sleep(std::time::Duration::from_millis(ASYNC_POLL_AWAIT_MS)).await;
			continue;
		};

		// Prepare input
		let mut input = BTreeMap::new();
		for (name, value) in job.input {
			match value {
				AttrData::Blob { bucket, key } => input.insert(
					name,
					PipeData::Blob {
						source: BytesProcessorBuilder::new(RawBytesSource::S3 { bucket, key }),
					},
				),

				// This should never fail, we handle all special cases above
				_ => input.insert(name, PipeData::try_from(value).unwrap()),
			};
		}

		// Set up context
		let mut conn = itemdb_client.new_connection().await.unwrap();
		let trans = conn.begin().await.unwrap();
		let context = CopperContext {
			stream_fragment_size: config.piper_stream_fragment_size,
			stream_channel_size: config.piper_stream_channel_size,
			job_id: job.job_id.as_str().into(),
			run_by_user: job.owned_by,
			itemdb_client: itemdb_client.clone(),
			objectstore_blob_bucket: config.piper_objectstore_storage_bucket.as_str().into(),
			objectstore_client: client.clone(),
			item_db_transaction: Mutex::new(trans),
		};

		// Run job
		let res = runner
			.run_job(context, job.pipeline, &job.job_id, input)
			.await;

		match res {
			Err(err) => handle_start_job_error(err, &job.job_id, &jobqueue_client).await,
			Ok(Err(error)) => handle_run_job_error(error, &job.job_id, &jobqueue_client).await,
			Ok(Ok(())) => handle_run_job_success(&job.job_id, &jobqueue_client).await,
		}
	}
}

async fn handle_start_job_error(
	err: StartJobError,
	job_id: &QueuedJobId,
	jobqueue_client: &PgJobQueueClient,
) {
	match err {
		StartJobError::BuildError(err) => {
			match jobqueue_client
				.builderror_job(job_id, &format!("{:?}", err))
				.await
			{
				Ok(()) => {}

				Err(BuildErrorJobError::DbError(error)) => {
					error!(
						message = "DB error while marking job `BuildError`",
						?job_id,
						?error
					);
				}

				Err(BuildErrorJobError::NotRunning) => {
					error!(
						message = "Tried to mark a job that isn't running as `BuildError`",
						?job_id
					);
				}
			}
		}
	}
}

async fn handle_run_job_error(
	error: RunNodeError,
	job_id: &QueuedJobId,
	jobqueue_client: &PgJobQueueClient,
) {
	info!(message = "Job failed", ?job_id, ?error);

	match jobqueue_client
		.fail_job_run(job_id, &format!("{}", error))
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

async fn handle_run_job_success(job_id: &QueuedJobId, jobqueue_client: &PgJobQueueClient) {
	info!(message = "Job finished successfully", ?job_id);

	match jobqueue_client.success_job(job_id).await {
		Ok(()) => {}

		Err(SuccessJobError::DbError(error)) => {
			error!(
				message = "DB error while marking job `Failed`",
				?job_id,
				?error
			);
		}

		Err(SuccessJobError::NotRunning) => {
			error!(
				message = "Tried to mark a job that isn't running as `Failed`",
				?job_id
			);
		}
	}
}
