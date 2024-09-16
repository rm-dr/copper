use async_trait::async_trait;
use reqwest::{header, Client, IntoUrl, StatusCode, Url};
use serde_json::json;

use super::{StoragedClient, StoragedRequestError};
use crate::{ClassId, ClassInfo, Transaction};

pub struct ReqwestStoragedClient {
	client: Client,
	storaged_url: Url,
	storaged_secret: String,
}

impl ReqwestStoragedClient {
	pub fn new(storaged_url: impl IntoUrl, storaged_secret: &str) -> Result<Self, reqwest::Error> {
		Ok(Self {
			client: Client::new(),
			storaged_url: storaged_url.into_url()?,
			storaged_secret: storaged_secret.to_string(),
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
					.join(&format!("/class/{}", i64::from(class_id)))
					.unwrap(),
			)
			.header(
				header::AUTHORIZATION,
				format!("Bearer {}", self.storaged_secret),
			)
			.send()
			.await
			.map_err(convert_error)?;

		match res.status() {
			StatusCode::OK => {
				let class: ClassInfo = res.json().await.map_err(convert_error)?;
				return Ok(Some(class));
			}

			x => {
				return Err(StoragedRequestError::GenericHttp {
					code: x.as_u16(),
					message: res.text().await.ok(),
				})
			}
		}
	}

	async fn apply_transaction(
		&self,
		transaction: Transaction,
	) -> Result<(), StoragedRequestError> {
		self.client
			.post(self.storaged_url.join("/transaction/apply").unwrap())
			.header(
				header::AUTHORIZATION,
				format!("Bearer {}", self.storaged_secret),
			)
			.json(&json!({
				"transaction": transaction
			}))
			.send()
			.await
			.map_err(convert_error)?;

		return Ok(());
	}
}
