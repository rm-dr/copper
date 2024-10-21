use async_trait::async_trait;
use reqwest::{header, Client, ClientBuilder, IntoUrl, StatusCode, Url};
use serde_json::json;

use super::{GenericRequestError, StoragedClient, StoragedRequestError};
use crate::{
	ApplyTransactionApiError, AttrDataStub, AttributeId, AttributeInfo, AttributeOptions, ClassId,
	ClassInfo, DatasetId, DatasetInfo, Transaction, UserId,
};

pub struct ReqwestStoragedClient {
	client: Client,
	storaged_url: Url,
	storaged_secret: String,
}

impl ReqwestStoragedClient {
	pub fn new(storaged_url: impl IntoUrl, storaged_secret: &str) -> Result<Self, reqwest::Error> {
		Ok(Self {
			// This might segfault if our ssl lib isn't linked correctly.
			// (this is why we use `rustls` everywhere, see cargo.toml)
			client: ClientBuilder::new().build()?,
			storaged_url: storaged_url.into_url()?,
			storaged_secret: storaged_secret.to_string(),
		})
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
	) -> Result<Result<DatasetId, GenericRequestError>, StoragedRequestError> {
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
			.map_err(Box::new)
			.map_err(|error| StoragedRequestError::RequestError { error })?;

		match res.status() {
			StatusCode::OK => {
				let ds: i64 = res
					.json()
					.await
					.map_err(Box::new)
					.map_err(|error| StoragedRequestError::RequestError { error })?;
				return Ok(Ok(ds.into()));
			}

			x => {
				return Ok(Err(GenericRequestError {
					code: x,
					message: res.text().await.ok(),
				}))
			}
		}
	}

	async fn get_dataset(
		&self,
		dataset: DatasetId,
	) -> Result<Result<Option<DatasetInfo>, GenericRequestError>, StoragedRequestError> {
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
			.map_err(Box::new)
			.map_err(|error| StoragedRequestError::RequestError { error })?;

		match res.status() {
			StatusCode::OK => {
				let ds: DatasetInfo = res
					.json()
					.await
					.map_err(Box::new)
					.map_err(|error| StoragedRequestError::RequestError { error })?;
				return Ok(Ok(Some(ds)));
			}

			x => {
				return Ok(Err(GenericRequestError {
					code: x,
					message: res.text().await.ok(),
				}))
			}
		}
	}

	async fn list_datasets(
		&self,
		owner: UserId,
	) -> Result<Result<Vec<DatasetInfo>, GenericRequestError>, StoragedRequestError> {
		let res = self
			.client
			.get(
				self.storaged_url
					.join(&format!("/dataset/owned_by/{}", i64::from(owner)))
					.unwrap(),
			)
			.header(
				header::AUTHORIZATION,
				format!("Bearer {}", self.storaged_secret),
			)
			.send()
			.await
			.map_err(Box::new)
			.map_err(|error| StoragedRequestError::RequestError { error })?;

		match res.status() {
			StatusCode::OK => {
				let ds: Vec<DatasetInfo> = res
					.json()
					.await
					.map_err(Box::new)
					.map_err(|error| StoragedRequestError::RequestError { error })?;
				return Ok(Ok(ds));
			}

			x => {
				return Ok(Err(GenericRequestError {
					code: x,
					message: res.text().await.ok(),
				}))
			}
		}
	}

	async fn rename_dataset(
		&self,
		dataset: DatasetId,
		new_name: &str,
	) -> Result<Result<(), GenericRequestError>, StoragedRequestError> {
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
			.map_err(Box::new)
			.map_err(|error| StoragedRequestError::RequestError { error })?;

		match res.status() {
			StatusCode::OK => {
				return Ok(Ok(()));
			}

			x => {
				return Ok(Err(GenericRequestError {
					code: x,
					message: res.text().await.ok(),
				}))
			}
		}
	}

	async fn delete_dataset(
		&self,
		dataset: DatasetId,
	) -> Result<Result<(), GenericRequestError>, StoragedRequestError> {
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
			.map_err(Box::new)
			.map_err(|error| StoragedRequestError::RequestError { error })?;

		match res.status() {
			StatusCode::OK => {
				return Ok(Ok(()));
			}

			x => {
				return Ok(Err(GenericRequestError {
					code: x,
					message: res.text().await.ok(),
				}))
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
	) -> Result<Result<ClassId, GenericRequestError>, StoragedRequestError> {
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
			.map_err(Box::new)
			.map_err(|error| StoragedRequestError::RequestError { error })?;

		match res.status() {
			StatusCode::OK => {
				let ds: i64 = res
					.json()
					.await
					.map_err(Box::new)
					.map_err(|error| StoragedRequestError::RequestError { error })?;
				return Ok(Ok(ds.into()));
			}

			x => {
				return Ok(Err(GenericRequestError {
					code: x,
					message: res.text().await.ok(),
				}))
			}
		}
	}

	async fn get_class(
		&self,
		class_id: ClassId,
	) -> Result<Result<Option<ClassInfo>, GenericRequestError>, StoragedRequestError> {
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
			.map_err(Box::new)
			.map_err(|error| StoragedRequestError::RequestError { error })?;

		match res.status() {
			StatusCode::OK => {
				let class: ClassInfo = res
					.json()
					.await
					.map_err(Box::new)
					.map_err(|error| StoragedRequestError::RequestError { error })?;
				return Ok(Ok(Some(class)));
			}

			x => {
				return Ok(Err(GenericRequestError {
					code: x,
					message: res.text().await.ok(),
				}))
			}
		}
	}

	async fn rename_class(
		&self,
		class: ClassId,
		new_name: &str,
	) -> Result<Result<(), GenericRequestError>, StoragedRequestError> {
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
			.map_err(Box::new)
			.map_err(|error| StoragedRequestError::RequestError { error })?;

		match res.status() {
			StatusCode::OK => {
				return Ok(Ok(()));
			}

			x => {
				return Ok(Err(GenericRequestError {
					code: x,
					message: res.text().await.ok(),
				}))
			}
		}
	}

	async fn del_class(
		&self,
		class: ClassId,
	) -> Result<Result<(), GenericRequestError>, StoragedRequestError> {
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
			.map_err(Box::new)
			.map_err(|error| StoragedRequestError::RequestError { error })?;

		match res.status() {
			StatusCode::OK => {
				return Ok(Ok(()));
			}

			x => {
				return Ok(Err(GenericRequestError {
					code: x,
					message: res.text().await.ok(),
				}))
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
	) -> Result<Result<AttributeId, GenericRequestError>, StoragedRequestError> {
		let res = self
			.client
			.post(
				self.storaged_url
					.join(&format!("/class/{}/attribute", i64::from(in_class)))
					.unwrap(),
			)
			// TODO: expose structs
			.json(&json!({
				"data_type": with_type,
				"name": name.to_string(),
				"options": options
			}))
			.header(
				header::AUTHORIZATION,
				format!("Bearer {}", self.storaged_secret),
			)
			.send()
			.await
			.map_err(Box::new)
			.map_err(|error| StoragedRequestError::RequestError { error })?;

		match res.status() {
			StatusCode::OK => {
				let ds: i64 = res
					.json()
					.await
					.map_err(Box::new)
					.map_err(|error| StoragedRequestError::RequestError { error })?;
				return Ok(Ok(ds.into()));
			}

			x => {
				return Ok(Err(GenericRequestError {
					code: x,
					message: res.text().await.ok(),
				}))
			}
		}
	}

	async fn get_attribute(
		&self,
		attribute: AttributeId,
	) -> Result<Result<Option<AttributeInfo>, GenericRequestError>, StoragedRequestError> {
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
			.map_err(Box::new)
			.map_err(|error| StoragedRequestError::RequestError { error })?;

		match res.status() {
			StatusCode::OK => {
				let attribute: AttributeInfo = res
					.json()
					.await
					.map_err(Box::new)
					.map_err(|error| StoragedRequestError::RequestError { error })?;
				return Ok(Ok(Some(attribute)));
			}

			x => {
				return Ok(Err(GenericRequestError {
					code: x,
					message: res.text().await.ok(),
				}))
			}
		}
	}

	async fn rename_attribute(
		&self,
		attribute: AttributeId,
		new_name: &str,
	) -> Result<Result<(), GenericRequestError>, StoragedRequestError> {
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
			.map_err(Box::new)
			.map_err(|error| StoragedRequestError::RequestError { error })?;

		match res.status() {
			StatusCode::OK => {
				return Ok(Ok(()));
			}

			x => {
				return Ok(Err(GenericRequestError {
					code: x,
					message: res.text().await.ok(),
				}))
			}
		}
	}

	async fn del_attribute(
		&self,
		attribute: AttributeId,
	) -> Result<Result<(), GenericRequestError>, StoragedRequestError> {
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
			.map_err(Box::new)
			.map_err(|error| StoragedRequestError::RequestError { error })?;

		match res.status() {
			StatusCode::OK => {
				return Ok(Ok(()));
			}

			x => {
				return Ok(Err(GenericRequestError {
					code: x,
					message: res.text().await.ok(),
				}))
			}
		}
	}

	//
	// MARK: other
	//

	async fn apply_transaction(
		&self,
		transaction: Transaction,
	) -> Result<Result<(), ApplyTransactionApiError>, StoragedRequestError> {
		let res = self
			.client
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
			.map_err(Box::new)
			.map_err(|error| StoragedRequestError::RequestError { error })?;

		match res.status() {
			StatusCode::OK => {
				return Ok(Ok(()));
			}

			StatusCode::BAD_REQUEST => {
				let x: ApplyTransactionApiError = res
					.json()
					.await
					.map_err(Box::new)
					.map_err(|error| StoragedRequestError::RequestError { error })?;

				return Ok(Err(x));
			}

			_ => {
				// TODO: handle 500
				unreachable!("Got unexpected status code")
			}
		}
	}
}
