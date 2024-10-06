use async_trait::async_trait;
use copper_storaged::AttrData;
use reqwest::{header, Client, IntoUrl, Url};
use serde_json::json;
use smartstring::{LazyCompact, SmartString};
use std::collections::BTreeMap;

use crate::json::PipelineJson;

use super::{PipelinedClient, PipelinedRequestError};

pub struct ReqwestPipelineClient {
	client: Client,
	pipelined_url: Url,
	pipelined_secret: String,
}

impl ReqwestPipelineClient {
	pub fn new(
		pipelined_url: impl IntoUrl,
		pipelined_secret: &str,
	) -> Result<Self, reqwest::Error> {
		Ok(Self {
			client: Client::new(),
			pipelined_url: pipelined_url.into_url()?,
			pipelined_secret: pipelined_secret.to_string(),
		})
	}
}

fn convert_error(e: reqwest::Error) -> PipelinedRequestError {
	if let Some(status) = e.status() {
		PipelinedRequestError::GenericHttp {
			code: status,
			message: Some(e.to_string()),
		}
	} else {
		PipelinedRequestError::Other { error: Box::new(e) }
	}
}

#[async_trait]
impl PipelinedClient for ReqwestPipelineClient {
	async fn run_pipeline(
		&self,
		pipeline: &PipelineJson,
		job_id: &str,
		input: &BTreeMap<SmartString<LazyCompact>, AttrData>,
	) -> Result<(), PipelinedRequestError> {
		self.client
			.post(self.pipelined_url.join("/pipeline/run").unwrap())
			.header(
				header::AUTHORIZATION,
				format!("Bearer {}", self.pipelined_secret),
			)
			.json(&json!({
				"pipeline": pipeline,
				"job_id": job_id,
				"input": input
			}))
			.send()
			.await
			.map_err(convert_error)?;

		return Ok(());
	}
}
