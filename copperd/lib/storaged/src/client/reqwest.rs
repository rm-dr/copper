use async_trait::async_trait;
use copper_edged::UserId;
use reqwest::{header, Client, IntoUrl, StatusCode, Url};
use serde_json::json;

use super::{StoragedClient, StoragedRequestError};
use crate::{
	AttrDataStub, AttributeId, AttributeInfo, AttributeOptions, ClassId, ClassInfo, DatasetId,
	DatasetInfo, Transaction,
};

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
			code: status,
			message: Some(e.to_string()),
		}
	} else {
		StoragedRequestError::Other { error: Box::new(e) }
	}
}

#[async_trait]
impl StoragedClient for ReqwestStoragedClient {
	//
	// MARK: dataset
	//

	async fn add_dataset(
		&self,
		name: &str,
		owner: UserId,
	) -> Result<DatasetId, StoragedRequestError> {
		let res = self
			.client
			.post(self.storaged_url.join("/dataset").unwrap())
			// TODO: expose structs
			.json(&json!({
				"owner": i64::from(owner),
				"name": name.to_string()
			}))
			.header(
				header::AUTHORIZATION,
				format!("Bearer {}", self.storaged_secret),
			)
			.send()
			.await
			.map_err(convert_error)?;

		match res.status() {
			StatusCode::OK => {
				let ds: i64 = res.json().await.map_err(convert_error)?;
				return Ok(ds.into());
			}

			x => {
				return Err(StoragedRequestError::GenericHttp {
					code: x,
					message: res.text().await.ok(),
				})
			}
		}
	}

	async fn get_dataset(
		&self,
		dataset: DatasetId,
	) -> Result<Option<DatasetInfo>, StoragedRequestError> {
		let res = self
			.client
			.get(
				self.storaged_url
					.join(&format!("/dataset/{}", i64::from(dataset)))
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
				let ds: DatasetInfo = res.json().await.map_err(convert_error)?;
				return Ok(Some(ds));
			}

			x => {
				return Err(StoragedRequestError::GenericHttp {
					code: x,
					message: res.text().await.ok(),
				})
			}
		}
	}

	async fn rename_dataset(
		&self,
		dataset: DatasetId,
		new_name: &str,
	) -> Result<(), StoragedRequestError> {
		let res = self
			.client
			.patch(
				self.storaged_url
					.join(&format!("/dataset/{}", i64::from(dataset)))
					.unwrap(),
			)
			.json(&json!({
				"new_name": new_name.to_string()
			}))
			.header(
				header::AUTHORIZATION,
				format!("Bearer {}", self.storaged_secret),
			)
			.send()
			.await
			.map_err(convert_error)?;

		match res.status() {
			StatusCode::OK => {
				return Ok(());
			}

			x => {
				return Err(StoragedRequestError::GenericHttp {
					code: x,
					message: res.text().await.ok(),
				})
			}
		}
	}

	async fn delete_dataset(&self, dataset: DatasetId) -> Result<(), StoragedRequestError> {
		let res = self
			.client
			.delete(
				self.storaged_url
					.join(&format!("/dataset/{}", i64::from(dataset)))
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
				return Ok(());
			}

			x => {
				return Err(StoragedRequestError::GenericHttp {
					code: x,
					message: res.text().await.ok(),
				})
			}
		}
	}
	//
	// MARK: class
	//

	async fn add_class(
		&self,
		in_dataset: DatasetId,
		name: &str,
	) -> Result<ClassId, StoragedRequestError> {
		let res = self
			.client
			.post(
				self.storaged_url
					.join(&format!("/dataset/{}/class", i64::from(in_dataset)))
					.unwrap(),
			)
			// TODO: expose structs
			.json(&json!({
				"name": name.to_string()
			}))
			.header(
				header::AUTHORIZATION,
				format!("Bearer {}", self.storaged_secret),
			)
			.send()
			.await
			.map_err(convert_error)?;

		match res.status() {
			StatusCode::OK => {
				let ds: i64 = res.json().await.map_err(convert_error)?;
				return Ok(ds.into());
			}

			x => {
				return Err(StoragedRequestError::GenericHttp {
					code: x,
					message: res.text().await.ok(),
				})
			}
		}
	}

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
					code: x,
					message: res.text().await.ok(),
				})
			}
		}
	}

	async fn rename_class(
		&self,
		class: ClassId,
		new_name: &str,
	) -> Result<(), StoragedRequestError> {
		let res = self
			.client
			.patch(
				self.storaged_url
					.join(&format!("/class/{}", i64::from(class)))
					.unwrap(),
			)
			.json(&json!({
				"new_name": new_name.to_string()
			}))
			.header(
				header::AUTHORIZATION,
				format!("Bearer {}", self.storaged_secret),
			)
			.send()
			.await
			.map_err(convert_error)?;

		match res.status() {
			StatusCode::OK => {
				return Ok(());
			}

			x => {
				return Err(StoragedRequestError::GenericHttp {
					code: x,
					message: res.text().await.ok(),
				})
			}
		}
	}

	async fn del_class(&self, class: ClassId) -> Result<(), StoragedRequestError> {
		let res = self
			.client
			.delete(
				self.storaged_url
					.join(&format!("/class/{}", i64::from(class)))
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
				return Ok(());
			}

			x => {
				return Err(StoragedRequestError::GenericHttp {
					code: x,
					message: res.text().await.ok(),
				})
			}
		}
	}

	//
	// MARK: attribute
	//

	async fn add_attribute(
		&self,
		in_class: ClassId,
		name: &str,
		with_type: AttrDataStub,
		options: AttributeOptions,
	) -> Result<AttributeId, StoragedRequestError> {
		let res = self
			.client
			.post(
				self.storaged_url
					.join(&format!("/class/{}/attribute", i64::from(in_class)))
					.unwrap(),
			)
			// TODO: expose structs
			.json(&json!({
				"type": with_type,
				"name": name.to_string(),
				"options": options
			}))
			.header(
				header::AUTHORIZATION,
				format!("Bearer {}", self.storaged_secret),
			)
			.send()
			.await
			.map_err(convert_error)?;

		match res.status() {
			StatusCode::OK => {
				let ds: i64 = res.json().await.map_err(convert_error)?;
				return Ok(ds.into());
			}

			x => {
				return Err(StoragedRequestError::GenericHttp {
					code: x,
					message: res.text().await.ok(),
				})
			}
		}
	}

	async fn get_attribute(
		&self,
		attribute: AttributeId,
	) -> Result<Option<AttributeInfo>, StoragedRequestError> {
		let res = self
			.client
			.get(
				self.storaged_url
					.join(&format!("/attribute/{}", i64::from(attribute)))
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
				let attribute: AttributeInfo = res.json().await.map_err(convert_error)?;
				return Ok(Some(attribute));
			}

			x => {
				return Err(StoragedRequestError::GenericHttp {
					code: x,
					message: res.text().await.ok(),
				})
			}
		}
	}

	async fn rename_attribute(
		&self,
		attribute: AttributeId,
		new_name: &str,
	) -> Result<(), StoragedRequestError> {
		let res = self
			.client
			.patch(
				self.storaged_url
					.join(&format!("/attribute/{}", i64::from(attribute)))
					.unwrap(),
			)
			.json(&json!({
				"new_name": new_name.to_string()
			}))
			.header(
				header::AUTHORIZATION,
				format!("Bearer {}", self.storaged_secret),
			)
			.send()
			.await
			.map_err(convert_error)?;

		match res.status() {
			StatusCode::OK => {
				return Ok(());
			}

			x => {
				return Err(StoragedRequestError::GenericHttp {
					code: x,
					message: res.text().await.ok(),
				})
			}
		}
	}

	async fn del_attribute(&self, attribute: AttributeId) -> Result<(), StoragedRequestError> {
		let res = self
			.client
			.delete(
				self.storaged_url
					.join(&format!("/attribute/{}", i64::from(attribute)))
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
				return Ok(());
			}

			x => {
				return Err(StoragedRequestError::GenericHttp {
					code: x,
					message: res.text().await.ok(),
				})
			}
		}
	}

	//
	// MARK: other
	//

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
