use std::{path::PathBuf, sync::Arc};

use serde::{Deserialize, Serialize};
use ufo_db_metastore::data::MetastoreDataStub;
use ufo_pipeline::labels::{PipelineLabel, PipelineNodeLabel};
use ufo_pipeline_nodes::data::UFOData;

#[derive(Deserialize, Serialize, Debug)]
pub struct RunnerStatus {
	pub queued_jobs: usize,
	pub finished_jobs: usize,
	pub running_jobs: Vec<RunningJobStatus>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RunningJobStatus {
	pub job_id: u128,
	pub pipeline: PipelineLabel,
	pub node_status: Vec<RunningNodeStatus>,

	// This pipeline's input, converted to a pretty string.
	// Context-dependent.
	pub input_exemplar: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CompletedJobStatus {
	pub job_id: u128,
	pub pipeline: PipelineLabel,
	pub error: Option<String>,
	pub input_exemplar: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RunningNodeStatus {
	pub name: PipelineNodeLabel,
	pub state: RunningNodeState,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "type")]
pub enum RunningNodeState {
	Pending { message: String },
	Running,
	Done,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AddJobParams {
	pub pipeline: PipelineLabel,
	pub input: Vec<PipelineInputData>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "type")]
pub enum AddJobResult {
	Ok, // TODO: return job id
	BadPipeline { pipeline: PipelineLabel },
	InvalidNumberOfArguments { got: usize, expected: usize },
	InvalidInputType { bad_input_idx: usize },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum PipelineInputData {
	/// Typed, unset data
	None(MetastoreDataStub),

	/// A block of text
	Text(String),

	/// A filesystem path
	Path(PathBuf),

	/// An integer
	Integer(i64),

	/// A positive integer
	PositiveInteger(u64),

	/// A boolean
	Boolean(bool),

	/// A float
	Float(f64),
	/*
	/// A checksum
	#[serde(skip)]
	Hash {
		format: HashType,
		data: Arc<Vec<u8>>,
	},

	/// Small binary data.
	/// This will be stored in the metadata db.
	#[serde(skip)]
	Binary {
		/// This data's media type
		format: MimeType,

		/// The data
		data: Arc<Vec<u8>>,
	},

	/// Big binary data.
	/// This will be stored in the blob store.
	#[serde(skip)]
	Blob {
		/// This data's media type
		format: MimeType,

		/// A receiver that provides data
		fragment: Arc<Vec<u8>>,

		/// Is this the last fragment?
		is_last: bool,
	},

	#[serde(skip)]
	Reference {
		/// The item class this
		class: ClassHandle,

		/// The item
		item: ItemHandle,
	},
	*/
}

impl From<PipelineInputData> for UFOData {
	fn from(value: PipelineInputData) -> Self {
		match value {
			PipelineInputData::None(t) => UFOData::None(t),
			PipelineInputData::Text(s) => UFOData::Text(Arc::new(s)),
			PipelineInputData::Path(p) => UFOData::Path(Arc::new(p)),
			PipelineInputData::Integer(i) => UFOData::Integer(i),
			PipelineInputData::PositiveInteger(u) => UFOData::PositiveInteger(u),
			PipelineInputData::Boolean(b) => UFOData::Boolean(b),
			PipelineInputData::Float(f) => UFOData::Float(f),
		}
	}
}

impl PipelineInputData {
	pub fn get_type(&self) -> MetastoreDataStub {
		match self {
			PipelineInputData::None(t) => t.clone(),
			PipelineInputData::Text(_) => MetastoreDataStub::Text,
			PipelineInputData::Path(_) => MetastoreDataStub::Path,
			PipelineInputData::Integer(_) => MetastoreDataStub::Integer,
			PipelineInputData::PositiveInteger(_) => MetastoreDataStub::PositiveInteger,
			PipelineInputData::Boolean(_) => MetastoreDataStub::Boolean,
			PipelineInputData::Float(_) => MetastoreDataStub::Float,
		}
	}
}
