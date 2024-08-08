use ufo_api::upload::UploadStartResult;
use ufo_api::{
	pipeline::{AddJobParams, NodeInfo, PipelineInfo},
	runner::RunnerStatus,
};
use ufo_pipeline::labels::{PipelineLabel, PipelineNodeLabel};
use url::Url;

use super::errors::UfoApiError;
use super::upload::UfoApiUploadJob;

pub struct UfoApiClient {
	pub(super) host: Url,
	pub(super) client: reqwest::blocking::Client,
}

impl UfoApiClient {
	pub fn new(host: Url) -> Self {
		Self {
			host,
			client: reqwest::blocking::Client::new(),
		}
	}

	pub fn get_host(&self) -> &Url {
		&self.host
	}
}

impl UfoApiClient {
	pub fn get_status(&self) -> RunnerStatus {
		let resp = self
			.client
			.get(self.host.join("status").unwrap())
			.send()
			.unwrap();
		serde_json::from_str(&resp.text().unwrap()).unwrap()
	}

	pub fn get_pipelines(&self) -> Vec<PipelineLabel> {
		let resp = self
			.client
			// TODO: url encode?
			.get(self.host.join("pipelines").unwrap())
			.send()
			.unwrap();
		serde_json::from_str(&resp.text().unwrap()).unwrap()
	}

	pub fn get_pipeline(&self, pipeline_name: &PipelineLabel) -> Option<PipelineInfo> {
		let resp = self
			.client
			.get(
				self.host
					.join("pipelines/")
					.unwrap()
					.join(pipeline_name.into())
					.unwrap(),
			)
			.send()
			.unwrap();
		let t = resp.text();
		serde_json::from_str(&t.unwrap()).unwrap()
	}

	pub fn get_pipeline_node(
		&self,
		pipeline_name: &PipelineLabel,
		node_name: &PipelineNodeLabel,
	) -> Option<NodeInfo> {
		let resp = self
			.client
			.get(
				self.host
					.join("pipelines/")
					.unwrap()
					.join(&format!("{}/", pipeline_name))
					.unwrap()
					.join(node_name.into())
					.unwrap(),
			)
			.send()
			.unwrap();
		serde_json::from_str(&resp.text().unwrap()).unwrap()
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

	pub fn add_job(&self, job: AddJobParams) {
		self.client
			.post(self.host.join("add_job").unwrap())
			.json(&job)
			.send()
			.unwrap();
	}
}
