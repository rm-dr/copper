use std::{collections::BTreeMap, error::Error, fmt::Display};

use async_trait::async_trait;
use copper_storaged::{AttrData, UserId};
use reqwest::StatusCode;
use smartstring::{LazyCompact, SmartString};

use crate::{json::PipelineJson, structs::JobInfoList};

#[derive(Debug)]
pub enum PipelinedRequestError {
	GenericHttp {
		code: StatusCode,
		message: Option<String>,
	},

	Other {
		error: Box<dyn Error + Sync + Send + 'static>,
	},
}

impl Display for PipelinedRequestError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::GenericHttp { code, message } => {
				if let Some(m) = message {
					write!(f, "Request failed with code {code}: {m}")
				} else {
					write!(f, "Request failed with code {code}")
				}
			}
			Self::Other { .. } => write!(f, "request failed"),
		}
	}
}

impl Error for PipelinedRequestError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::Other { error } => Some(error.as_ref()),
			_ => None,
		}
	}
}

#[async_trait]
pub trait PipelinedClient: Send + Sync {
	async fn run_pipeline(
		&self,
		pipeline: &PipelineJson,
		job_id: &str,
		input: &BTreeMap<SmartString<LazyCompact>, AttrData>,
		as_user: UserId,
	) -> Result<(), PipelinedRequestError>;

	async fn list_user_jobs(
		&self,
		user: UserId,
		skip: usize,
		count: usize,
	) -> Result<JobInfoList, PipelinedRequestError>;
}
