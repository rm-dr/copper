use copper_pipelined::{
	base::{NodeDispatcher, PipelineData, PipelineJobContext, RunNodeError},
	json::PipelineJson,
	structs::JobInfoState,
};
use copper_storaged::UserId;
use smartstring::{LazyCompact, SmartString};
use std::{
	collections::{BTreeMap, VecDeque},
	error::Error,
	fmt::Display,
};
use time::OffsetDateTime;
use tokio::task::{JoinError, JoinSet};
use tracing::{debug, info};

use super::job::PipelineBuildError;
use crate::pipeline::job::PipelineJob;

//
// MARK: Errors
//

#[derive(Debug)]
pub enum AddJobError {
	/// We tried to create a job with an id that already exists
	AlreadyExists,

	/// We tried to add a job, but the queue is full
	QueueFull,
}

impl Display for AddJobError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::AlreadyExists => write!(f, "a job with this id already exists"),
			Self::QueueFull => write!(f, "job queue is full"),
		}
	}
}

impl Error for AddJobError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		None
	}
}

//
// MARK: helpers
//

pub struct PipelineRunnerOptions {
	/// The maximum number of jobs we'll run at once
	pub max_running_jobs: usize,

	/// The size of the job queue
	pub job_queue_size: usize,

	/// The size of the job log
	pub job_log_size: usize,
}

pub enum JobState<ContextType> {
	Queued { context: ContextType },
	Running,
	Success,
	Failed,
	BuildError(PipelineBuildError),
}

impl<T> From<&JobState<T>> for JobInfoState {
	fn from(state: &JobState<T>) -> Self {
		match &state {
			JobState::Queued { .. } => JobInfoState::Queued,
			JobState::Running => JobInfoState::Running,
			JobState::Failed => JobInfoState::Failed,
			JobState::Success => JobInfoState::Success,
			JobState::BuildError(err) => JobInfoState::BuildError {
				message: format!("{err}"),
			},
		}
	}
}

impl<ContextType> JobState<ContextType> {
	/// If this is `Self::Queued`, return `context` and set to `Self::Running`.
	/// Otherwise, do nothing and return `None`.
	fn start(&mut self) -> Option<ContextType> {
		match self {
			Self::Queued { .. } => {
				let s = std::mem::replace(self, Self::Running);
				match s {
					Self::Queued { context } => return Some(context),
					_ => unreachable!(),
				}
			}

			_ => None,
		}
	}
}

pub struct JobEntry<DataType: PipelineData, ContextType: PipelineJobContext<DataType>> {
	pub id: SmartString<LazyCompact>,
	pub state: JobState<ContextType>,
	pub owner: UserId,

	pub pipeline: PipelineJson,
	pub inputs: BTreeMap<SmartString<LazyCompact>, DataType>,

	pub added_at: OffsetDateTime,

	pub started_at: Option<OffsetDateTime>,
	pub finished_at: Option<OffsetDateTime>,
}

//
// MARK: Runner
//

pub struct PipelineRunner<DataType: PipelineData, ContextType: PipelineJobContext<DataType>> {
	config: PipelineRunnerOptions,
	dispatcher: NodeDispatcher<DataType, ContextType>,

	jobs: BTreeMap<SmartString<LazyCompact>, JobEntry<DataType, ContextType>>,

	/// Jobs owned by each user.
	/// Queues are ordered by add time.
	/// Jobs are always added to the back of the deque.
	jobs_by_user: BTreeMap<UserId, VecDeque<SmartString<LazyCompact>>>,

	/// Jobs that are queued to run.
	/// These are are ordered by add time.
	/// Jobs are always added to the back of the deque.
	queued_jobs: VecDeque<SmartString<LazyCompact>>,
	running_jobs: VecDeque<SmartString<LazyCompact>>,
	finished_jobs: VecDeque<SmartString<LazyCompact>>,

	/// Jobs that are running right now
	#[allow(clippy::type_complexity)]
	tasks: JoinSet<(SmartString<LazyCompact>, Result<(), RunNodeError<DataType>>)>,
}

impl<DataType: PipelineData, ContextType: PipelineJobContext<DataType>>
	PipelineRunner<DataType, ContextType>
{
	/// Initialize a new runner
	pub fn new(config: PipelineRunnerOptions) -> Self {
		Self {
			dispatcher: NodeDispatcher::new(),

			tasks: JoinSet::new(),
			jobs: BTreeMap::new(),
			jobs_by_user: BTreeMap::new(),

			queued_jobs: VecDeque::with_capacity(config.job_queue_size),
			running_jobs: VecDeque::with_capacity(config.max_running_jobs),
			finished_jobs: VecDeque::with_capacity(config.job_log_size),

			config,
		}
	}

	/// Add a job to this runner's queue
	pub fn add_job(
		&mut self,
		context: ContextType,
		pipeline: PipelineJson,
		job_id: &str,
		owner: UserId,
		inputs: BTreeMap<SmartString<LazyCompact>, DataType>,
	) -> Result<(), AddJobError> {
		if self.jobs.contains_key(job_id) {
			debug!(message = "Job not added, already exists", job_id);
			return Err(AddJobError::AlreadyExists);
		}

		if self.queued_jobs.len() >= self.config.job_queue_size {
			debug!(message = "Job not added, queue full", job_id);
			return Err(AddJobError::QueueFull);
		}

		info!(message = "Adding job to queue", job_id);
		self.jobs.insert(
			job_id.into(),
			JobEntry {
				id: job_id.into(),
				state: JobState::Queued { context },
				owner,

				pipeline,
				inputs,

				added_at: OffsetDateTime::now_utc(),
				started_at: None,
				finished_at: None,
			},
		);

		self.queued_jobs.push_back(job_id.into());
		if !self.jobs_by_user.contains_key(&owner) {
			self.jobs_by_user.insert(owner, VecDeque::new());
		}
		self.jobs_by_user
			.get_mut(&owner)
			.unwrap()
			.push_back(job_id.into());

		return Ok(());
	}

	pub async fn run(&mut self) -> Result<(), JoinError> {
		//
		// Process finished jobs
		//
		while let Some(res) = self.tasks.try_join_next() {
			let res = res?;

			// Make sure job log stays within size limit
			while self.finished_jobs.len() >= self.config.job_log_size {
				let j = self.finished_jobs.pop_front().unwrap();
				let j = self.jobs.remove(&j).unwrap();
				let jbu = self.jobs_by_user.get_mut(&j.owner).unwrap();
				let idx = jbu.iter().position(|x| x == &j.id).unwrap();
				jbu.remove(idx);
			}

			let job = self.jobs.get_mut(&res.0).unwrap();
			job.finished_at = Some(OffsetDateTime::now_utc());

			self.finished_jobs.push_back(job.id.clone());
			self.running_jobs.remove(
				self.running_jobs
					.iter()
					.enumerate()
					.find(|(_, id)| job.id == **id)
					.unwrap()
					.0,
			);

			if let Err(err) = res.1 {
				job.state = JobState::Failed;

				debug!(
					message = "Job failed",
					job_id = ?res.0,

					added_at = ?job.added_at,
					started_at = ?job.started_at,
					run_time = ?(job.finished_at.unwrap() - job.started_at.unwrap()),
					error = format!("{err:?}")
				);
			} else {
				job.state = JobState::Success;

				debug!(
					message = "Job finished with no errors",
					job_id = ?res.0,

					added_at = ?job.added_at,
					started_at = ?job.started_at,
					run_time = ?(job.finished_at.unwrap() - job.started_at.unwrap()),
				);
			}
		}

		//
		// Start new jobs, if there is space in the set
		// and jobs in the queue.
		//
		while self.tasks.len() < self.config.max_running_jobs && !self.queued_jobs.is_empty() {
			let queued_job_id = self.queued_jobs.pop_front().unwrap();
			let job = self.jobs.get_mut(&queued_job_id).unwrap();
			let context = job.state.start().unwrap();
			job.started_at = Some(OffsetDateTime::now_utc());

			debug!(
				message = "Starting job",
				job_id = ?queued_job_id,
				added_at = ?job.added_at,
				running_jobs = self.tasks.len(),
				max_running_jobs = self.config.max_running_jobs,
				queued_jobs = self.queued_jobs.len() + 1
			);

			let res = PipelineJob::<DataType, ContextType>::new(
				&self.dispatcher,
				&job.id,
				job.inputs.clone(),
				&job.pipeline,
			);

			match res {
				Ok(job) => {
					self.running_jobs.push_back(queued_job_id.clone());
					self.tasks.spawn(async {
						// TODO: handle error
						let x = job.run(context).await;
						(queued_job_id, x)
					});
				}

				Err(err) => {
					debug!(
						message = "Could not start job, invalid pipeline",
						job_id = ?queued_job_id,
						error = ?err
					);

					job.finished_at = Some(OffsetDateTime::now_utc());
					job.state = JobState::BuildError(err);

					// Make sure job log stays within size limit
					while self.finished_jobs.len() >= self.config.job_log_size {
						let j = self.finished_jobs.pop_front().unwrap();
						let j = self.jobs.remove(&j).unwrap();
						let jbu = self.jobs_by_user.get_mut(&j.owner).unwrap();
						let idx = jbu.iter().position(|x| x == &j.id).unwrap();
						jbu.remove(idx);
					}

					self.finished_jobs.push_back(queued_job_id);
				}
			}
		}

		return Ok(());
	}
}

impl<DataType: PipelineData, ContextType: PipelineJobContext<DataType>>
	PipelineRunner<DataType, ContextType>
{
	/// Get this runner's node dispatcher
	pub fn mut_dispatcher(&mut self) -> &mut NodeDispatcher<DataType, ContextType> {
		&mut self.dispatcher
	}

	/// Get this runner's queued jobs
	pub fn jobs_by_user(&self, user: UserId) -> Option<&VecDeque<SmartString<LazyCompact>>> {
		self.jobs_by_user.get(&user)
	}

	pub fn get_job(&self, job_id: &str) -> Option<&JobEntry<DataType, ContextType>> {
		self.jobs.get(job_id)
	}
}
