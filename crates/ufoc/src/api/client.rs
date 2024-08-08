use reqwest::StatusCode;
use ufo_api::status::ServerStatus;
use ufo_api::upload::UploadStartResult;
use ufo_api::{
	pipeline::{AddJobParams, NodeInfo, PipelineInfo},
	status::RunnerStatus,
};
use ufo_pipeline::labels::{PipelineLabel, PipelineNodeLabel};
use url::Url;

use super::errors::UfoApiError;
use super::upload::UfoApiUploadJob;

pub struct UfoApiClient {
	pub(super) host: Url,
	pub(super) client: reqwest::blocking::Client,
	pub(super) request_body_limit: usize,
}

impl UfoApiClient {
	pub fn new(host: Url) -> Result<Self, UfoApiError> {
		let client = reqwest::blocking::Client::new();
		let server_status = client.get(host.join("/status").unwrap()).send()?;
		let server_status: ServerStatus = serde_json::from_str(&server_status.text()?)?;

		return Ok(Self {
			host,
			client,
			request_body_limit: server_status.request_body_limit,
		});
	}

	pub fn get_host(&self) -> &Url {
		&self.host
	}
}

impl UfoApiClient {
	pub fn get_server_status(&self) -> Result<ServerStatus, UfoApiError> {
		let resp = self.client.get(self.host.join("/status").unwrap()).send()?;
		return Ok(serde_json::from_str(&resp.text()?)?);
	}

	pub fn get_runner_status(&self) -> Result<RunnerStatus, UfoApiError> {
		let resp = self
			.client
			.get(self.host.join("/status/runner").unwrap())
			.send()?;

		return Ok(serde_json::from_str(&resp.text()?)?);
	}

	pub fn get_pipelines(&self) -> Result<Vec<PipelineLabel>, UfoApiError> {
		let resp = self
			.client
			.get(self.host.join("/pipelines").unwrap())
			.send()?;
		return Ok(serde_json::from_str(&resp.text()?)?);
	}

	pub fn get_pipeline(
		&self,
		pipeline_name: &PipelineLabel,
	) -> Result<Option<PipelineInfo>, UfoApiError> {
		let resp = self
			.client
			.get(
				self.host
					.join("/pipelines/")
					.unwrap()
					.join(pipeline_name.into())
					.unwrap(),
			)
			.send()?;

		return Ok(match resp.status() {
			StatusCode::NOT_FOUND => None,
			StatusCode::OK => serde_json::from_str(&resp.text()?)?,
			_ => unreachable!(),
		});
	}

	pub fn get_pipeline_node(
		&self,
		pipeline_name: &PipelineLabel,
		node_name: &PipelineNodeLabel,
	) -> Result<Option<NodeInfo>, UfoApiError> {
		let resp = self
			.client
			.get(
				self.host
					.join("/pipelines/")
					.unwrap()
					.join(&format!("{}/", pipeline_name))
					.unwrap()
					.join(node_name.into())
					.unwrap(),
			)
			.send()?;

		return Ok(match resp.status() {
			StatusCode::NOT_FOUND => None,
			StatusCode::OK => serde_json::from_str(&resp.text()?)?,
			_ => unreachable!(),
		});
	}

	pub fn new_upload_job(&self) -> Result<UfoApiUploadJob, UfoApiError> {
		let res = self
			.client
			.post(self.host.join("/upload/start").unwrap())
			.send()?;

		let res: UploadStartResult = serde_json::from_str(&res.text()?)?;
		return Ok(UfoApiUploadJob {
			api_client: self,
			upload_job_id: res.job_id,
		});
	}

	pub fn add_job(&self, job: AddJobParams) -> Result<(), UfoApiError> {
		self.client
			.post(self.host.join("/add_job").unwrap())
			.json(&job)
			.send()?;

		return Ok(());
	}
}
