use async_trait::async_trait;
use reqwest::{Client, IntoUrl, Url};
use serde_json::json;

use super::{StoragedClient, StoragedRequestError};
use crate::{ClassId, ClassInfo, Transaction};

pub struct ReqwestStoragedClient {
	client: Client,
	storaged_url: Url,
}

impl ReqwestStoragedClient {
	pub fn new(storaged_url: impl IntoUrl) -> Result<Self, reqwest::Error> {
		Ok(Self {
			client: Client::new(),
			storaged_url: storaged_url.into_url()?,
		})
	}
}

fn convert_error(e: reqwest::Error) -> StoragedRequestError {
	if let Some(status) = e.status() {
		StoragedRequestError::GenericHttp {
			code: status.as_u16(),
			message: Some(e.to_string()),
		}
	} else {
		StoragedRequestError::Other { error: Box::new(e) }
	}
}

#[async_trait]
impl StoragedClient for ReqwestStoragedClient {
	async fn get_class(
		&self,
		class_id: ClassId,
	) -> Result<Option<ClassInfo>, StoragedRequestError> {
		let res = self
			.client
			.get(
				self.storaged_url
					.join(&format!("/class/{}", u32::from(class_id)))
					.unwrap(),
			)
			.send()
			.await
			.map_err(convert_error)?;

		let class: ClassInfo = res.json().await.map_err(convert_error)?;
		return Ok(Some(class));
	}

	async fn apply_transaction(
		&self,
		transaction: Transaction,
	) -> Result<(), StoragedRequestError> {
		self.client
			.post(self.storaged_url.join("/apply").unwrap())
			.json(&json!({
				"transaction": transaction
			}))
			.send()
			.await
			.map_err(convert_error)?;

		return Ok(());
	}
}
