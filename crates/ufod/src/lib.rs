use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct RunnerStatus {
	pub queued_jobs: usize,
	pub finished_jobs: usize,
	pub running_jobs: Vec<RunningJobStatus>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RunningJobStatus {
	pub job_id: u128,
	pub pipeline: String,
	pub node_status: Vec<RunningNodeStatus>,

	// This pipeline's input, converted to a pretty string.
	// Context-dependent.
	pub input_exemplar: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CompletedJobStatus {
	pub job_id: u128,
	pub pipeline: String,
	pub error: Option<String>,
	pub input_exemplar: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RunningNodeStatus {
	pub name: String,
	pub state: RunningNodeState,
}

#[derive(Deserialize, Serialize, Debug)]
pub enum RunningNodeState {
	Pending(String),
	Running,
	Done,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AddJobParams {
	pub pipeline: String,
	pub input: PathBuf,
}
