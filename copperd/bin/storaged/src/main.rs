use api::RouterState;
use axum::Router;
use config::StoragedConfig;
use copper_util::load_env;
use database::postgres::{PgDatabaseClient, PgDatabaseOpenError};
use std::sync::Arc;
use tracing::{debug, error, info};

mod api;
mod config;
mod database;

async fn make_app(config: Arc<StoragedConfig>) -> Router {
	// Connect to database
	let db = match PgDatabaseClient::open(&config.storaged_db_addr).await {
		Ok(db) => db,
		Err(PgDatabaseOpenError::Database(e)) => {
			error!(message = "SQL error while opening database", err = ?e);
			std::process::exit(1);
		}
		Err(PgDatabaseOpenError::Migrate(e)) => {
			error!(message = "Migration error while opening database", err = ?e);
			std::process::exit(1);
		}
	};

	// Create app
	return api::router(RouterState {
		config: config.clone(),
		client: Arc::new(db),
	});
}

#[tokio::main]
async fn main() {
	let config = Arc::new(load_env::<StoragedConfig>());

	tracing_subscriber::fmt()
		.with_env_filter(config.storaged_loglevel.get_config())
		.without_time()
		.with_ansi(true)
		.init();

	debug!(message = "Loaded config from environment", ?config);

	let listener =
		match tokio::net::TcpListener::bind(config.storaged_server_addr.to_string()).await {
			Ok(x) => x,
			Err(e) => {
				match e.kind() {
					std::io::ErrorKind::AddrInUse => {
						error!(
							message = "Cannot bind to address, already in use",
							server_addr = config.storaged_server_addr.as_str()
						);
					}
					_ => {
						error!(message = "Error while migrating main database", err = ?e);
					}
				}

				std::process::exit(1);
			}
		};
	info!("listening on http://{}", listener.local_addr().unwrap());

	let app = make_app(config).await;

	match axum::serve(listener, app).await {
		Ok(_) => {}
		Err(e) => {
			error!(message = "Main loop exited with error", error = ?e)
		}
	};
}

#[cfg(test)]
mod tests {
	use super::*;
	use axum::{
		body::Body,
		http::{Method, Request, Response},
	};
	use copper_storaged::{
		AttrData, AttrDataStub, AttributeId, AttributeOptions, ClassId, DatasetId, Transaction,
		TransactionAction,
	};
	use copper_util::HashType;
	use serde::de::DeserializeOwned;
	use serde_json::json;
	use tower::Service;

	//
	// MARK: Helpers
	//

	async fn app_request(
		app: &mut Router,
		method: Method,
		url: &str,
		body: serde_json::Value,
	) -> Response<Body> {
		app.call(
			Request::builder()
				.method(method)
				.header(axum::http::header::CONTENT_TYPE, "application/json")
				.header(axum::http::header::AUTHORIZATION, "Bearer i_m_secret")
				.uri(url)
				.body(Body::from(serde_json::to_string(&body).unwrap()))
				.unwrap(),
		)
		.await
		.unwrap()
	}

	async fn create_dataset(app: &mut Router, name: &str) -> Response<Body> {
		app_request(
			app,
			Method::POST,
			"/dataset",
			json!({
				"name": name
			}),
		)
		.await
	}

	async fn rename_dataset(
		app: &mut Router,
		dataset: DatasetId,
		new_name: &str,
	) -> Response<Body> {
		app_request(
			app,
			Method::PATCH,
			&format!("/dataset/{}", i64::from(dataset)),
			json!({
				"new_name": new_name
			}),
		)
		.await
	}

	async fn create_class(app: &mut Router, dataset: DatasetId, name: &str) -> Response<Body> {
		app_request(
			app,
			Method::POST,
			&format!("/dataset/{}/class", i64::from(dataset)),
			json!({
				"name": name
			}),
		)
		.await
	}
	async fn rename_class(app: &mut Router, class: ClassId, new_name: &str) -> Response<Body> {
		app_request(
			app,
			Method::PATCH,
			&format!("/class/{}", i64::from(class)),
			json!({
				"new_name": new_name
			}),
		)
		.await
	}

	async fn create_attribute(
		app: &mut Router,
		class: ClassId,
		name: &str,
		data_type: AttrDataStub,
		options: AttributeOptions,
	) -> Response<Body> {
		app_request(
			app,
			Method::POST,
			&format!("/class/{}/attribute", i64::from(class)),
			json!({
				"name": name,
				"data_type": data_type,
				"options": options
			}),
		)
		.await
	}

	async fn rename_attribute(
		app: &mut Router,
		attribute: AttributeId,
		new_name: &str,
	) -> Response<Body> {
		app_request(
			app,
			Method::PATCH,
			&format!("/attribute/{}", i64::from(attribute)),
			json!({
				"new_name": new_name
			}),
		)
		.await
	}

	async fn apply_transaction(app: &mut Router, transaction: Transaction) -> Response<Body> {
		app_request(
			app,
			Method::POST,
			"/transaction/apply",
			json!({
				"transaction": transaction
			}),
		)
		.await
	}

	async fn response_json<T: DeserializeOwned>(resp: Response<Body>) -> T {
		let s = String::from_utf8(
			axum::body::to_bytes(resp.into_body(), usize::MAX)
				.await
				.unwrap()
				.to_vec(),
		)
		.unwrap();

		// This will panic if you try to call `response_json`
		// on a response that returns nothing.
		serde_json::from_str(&s).unwrap()
	}

	//
	// MARK: Test
	//

	#[tokio::test]
	async fn basic_crud() {
		tracing_subscriber::fmt()
			.without_time()
			.with_ansi(true)
			.init();

		// Set up config & create app
		let config = StoragedConfig {
			storaged_db_addr: "postgres://admin:admin@localhost/storaged".into(),
			storaged_server_addr: "".into(),
			storaged_request_body_limit: StoragedConfig::default_request_body_limit(),
			storaged_secret: "i_m_secret".into(),
			storaged_loglevel: Default::default(),
		};
		let mut app = make_app(Arc::new(config)).await;

		// Saved ids we intend do use later
		let test_dataset_id: DatasetId = 1.into();
		let test_dataset_two_id: DatasetId = 2.into();
		let class_covers_id: ClassId = 1.into();
		let class_audiofile_id: ClassId = 2.into();
		let attr_title_id: AttributeId = 3.into();

		//
		// MARK: Create datasets
		//

		// These requests should fail, invalid name
		{
			let response = create_dataset(&mut app, "").await;
			assert_eq!(response.status(), 400);
			assert_eq!(
				response_json::<String>(response).await,
				"name cannot be empty"
			);

			let response = create_dataset(&mut app, "  bad_dataset").await;
			assert_eq!(response.status(), 400);
			assert_eq!(
				response_json::<String>(response).await,
				"name cannot have leading or trailing whitespace"
			);

			let response = create_dataset(&mut app, "bad_dataset  ").await;
			assert_eq!(response.status(), 400);
			assert_eq!(
				response_json::<String>(response).await,
				"name cannot have leading or trailing whitespace"
			);

			let response = create_dataset(&mut app, "bad_dataset\t").await;
			assert_eq!(response.status(), 400);
			assert_eq!(
				response_json::<String>(response).await,
				"name cannot have leading or trailing whitespace"
			);

			let response = create_dataset(&mut app, "bad_dataset\n").await;
			assert_eq!(response.status(), 400);
			assert_eq!(
				response_json::<String>(response).await,
				"name cannot have leading or trailing whitespace"
			);
		}

		// These requests are perfectly fine
		{
			let response = create_dataset(&mut app, "test_dataset").await;
			assert_eq!(response.status(), 200);
			assert_eq!(response_json::<DatasetId>(response).await, test_dataset_id);

			let response = create_dataset(&mut app, "test_dataset_two").await;
			assert_eq!(response.status(), 200);
			assert_eq!(
				response_json::<DatasetId>(response).await,
				test_dataset_two_id
			);
		}

		// This request should fail, duplicate name
		{
			let response = create_dataset(&mut app, "test_dataset").await;
			assert_eq!(response.status(), 400);
			assert_eq!(
				response_json::<String>(response).await,
				"a dataset with this name already exists"
			);

			let response = rename_dataset(&mut app, test_dataset_id, "test_dataset_two").await;
			assert_eq!(response.status(), 400);
			assert_eq!(
				response_json::<String>(response).await,
				"a dataset with this name already exists"
			);
		}

		//
		// MARK: Create classes
		//

		// These requests should fail, invalid name
		{
			let response = create_class(&mut app, test_dataset_id, "").await;
			assert_eq!(response.status(), 400);
			assert_eq!(
				response_json::<String>(response).await,
				"name cannot be empty"
			);

			let response = create_class(&mut app, test_dataset_id, "  bad_class").await;
			assert_eq!(response.status(), 400);
			assert_eq!(
				response_json::<String>(response).await,
				"name cannot have leading or trailing whitespace"
			);

			let response = create_class(&mut app, test_dataset_id, "bad_class  ").await;
			assert_eq!(response.status(), 400);
			assert_eq!(
				response_json::<String>(response).await,
				"name cannot have leading or trailing whitespace"
			);

			let response = create_class(&mut app, test_dataset_id, "bad_class\t").await;
			assert_eq!(response.status(), 400);
			assert_eq!(
				response_json::<String>(response).await,
				"name cannot have leading or trailing whitespace"
			);

			let response = create_class(&mut app, test_dataset_id, "bad_class\n").await;
			assert_eq!(response.status(), 400);
			assert_eq!(
				response_json::<String>(response).await,
				"name cannot have leading or trailing whitespace"
			);
		}

		// This should fail, invalid dataset
		{
			let response = create_class(&mut app, 45.into(), "class_bad_dataset").await;
			assert_eq!(response.status(), 404);
		}

		// These requests is perfectly fine

		{
			let response = create_class(&mut app, test_dataset_id, "covers").await;
			assert_eq!(response.status(), 200);
			assert_eq!(response_json::<ClassId>(response).await, class_covers_id);

			let response = create_class(&mut app, test_dataset_id, "audiofile").await;
			assert_eq!(response.status(), 200);
			assert_eq!(response_json::<ClassId>(response).await, class_audiofile_id);
		}

		// These requests should fail, duplicate name
		{
			let response = create_class(&mut app, test_dataset_id, "covers").await;
			assert_eq!(response.status(), 400);
			assert_eq!(
				response_json::<String>(response).await,
				"a class with this name already exists"
			);

			let response = create_class(&mut app, test_dataset_id, "audiofile").await;
			assert_eq!(response.status(), 400);
			assert_eq!(
				response_json::<String>(response).await,
				"a class with this name already exists"
			);

			let response = rename_class(&mut app, class_covers_id, "audiofile").await;
			assert_eq!(response.status(), 400);
			assert_eq!(
				response_json::<String>(response).await,
				"a class with this name already exists"
			);
		}

		//
		// MARK: Create attributes
		//

		// These requests should fail, invalid name
		{
			let response = create_attribute(
				&mut app,
				class_covers_id,
				"",
				AttrDataStub::Text,
				AttributeOptions::default(),
			)
			.await;
			assert_eq!(response.status(), 400);
			assert_eq!(
				response_json::<String>(response).await,
				"name cannot be empty"
			);

			let response = create_attribute(
				&mut app,
				class_covers_id,
				"  bad_attr",
				AttrDataStub::Text,
				AttributeOptions::default(),
			)
			.await;
			assert_eq!(response.status(), 400);
			assert_eq!(
				response_json::<String>(response).await,
				"name cannot have leading or trailing whitespace"
			);

			let response = create_attribute(
				&mut app,
				class_covers_id,
				"bad_attr  ",
				AttrDataStub::Text,
				AttributeOptions::default(),
			)
			.await;
			assert_eq!(response.status(), 400);
			assert_eq!(
				response_json::<String>(response).await,
				"name cannot have leading or trailing whitespace"
			);

			let response = create_attribute(
				&mut app,
				class_covers_id,
				"bad_attr\t",
				AttrDataStub::Text,
				AttributeOptions::default(),
			)
			.await;
			assert_eq!(response.status(), 400);
			assert_eq!(
				response_json::<String>(response).await,
				"name cannot have leading or trailing whitespace"
			);

			let response = create_attribute(
				&mut app,
				class_covers_id,
				"bad_attr\n",
				AttrDataStub::Text,
				AttributeOptions::default(),
			)
			.await;
			assert_eq!(response.status(), 400);
			assert_eq!(
				response_json::<String>(response).await,
				"name cannot have leading or trailing whitespace"
			);
		}

		// These requests should fail, invalid class
		{
			let response = create_attribute(
				&mut app,
				45.into(),
				"attr_bad_class",
				AttrDataStub::Text,
				AttributeOptions::default(),
			)
			.await;
			assert_eq!(response.status(), 404);
		}

		// Create `cover` attributes
		{
			let response = create_attribute(
				&mut app,
				class_covers_id,
				"content_hash",
				AttrDataStub::Hash {
					hash_type: HashType::SHA256,
				},
				AttributeOptions::default(),
			)
			.await;
			assert_eq!(response.status(), 200);

			let response = create_attribute(
				&mut app,
				class_covers_id,
				"image",
				AttrDataStub::Blob,
				AttributeOptions::default(),
			)
			.await;
			assert_eq!(response.status(), 200);
		}

		// Create `audiofile` attributes
		{
			let response = create_attribute(
				&mut app,
				class_audiofile_id,
				"title",
				AttrDataStub::Text,
				AttributeOptions::default(),
			)
			.await;
			assert_eq!(response.status(), 200);
			assert_eq!(response_json::<AttributeId>(response).await, attr_title_id);

			let response = create_attribute(
				&mut app,
				class_audiofile_id,
				"album",
				AttrDataStub::Text,
				AttributeOptions::default(),
			)
			.await;
			assert_eq!(response.status(), 200);

			let response = create_attribute(
				&mut app,
				class_audiofile_id,
				"artist",
				AttrDataStub::Text,
				AttributeOptions::default(),
			)
			.await;
			assert_eq!(response.status(), 200);

			let response = create_attribute(
				&mut app,
				class_audiofile_id,
				"albumartist",
				AttrDataStub::Text,
				AttributeOptions::default(),
			)
			.await;
			assert_eq!(response.status(), 200);

			let response = create_attribute(
				&mut app,
				class_audiofile_id,
				"tracknumber",
				AttrDataStub::Text,
				AttributeOptions::default(),
			)
			.await;
			assert_eq!(response.status(), 200);

			let response = create_attribute(
				&mut app,
				class_audiofile_id,
				"year",
				AttrDataStub::Text,
				AttributeOptions::default(),
			)
			.await;
			assert_eq!(response.status(), 200);

			let response = create_attribute(
				&mut app,
				class_audiofile_id,
				"genre",
				AttrDataStub::Text,
				AttributeOptions::default(),
			)
			.await;
			assert_eq!(response.status(), 200);

			let response = create_attribute(
				&mut app,
				class_audiofile_id,
				"ISRC",
				AttrDataStub::Text,
				AttributeOptions::default(),
			)
			.await;
			assert_eq!(response.status(), 200);

			let response = create_attribute(
				&mut app,
				class_audiofile_id,
				"lyrics",
				AttrDataStub::Text,
				AttributeOptions::default(),
			)
			.await;
			assert_eq!(response.status(), 200);

			let response = create_attribute(
				&mut app,
				class_audiofile_id,
				"audio_data",
				AttrDataStub::Blob,
				AttributeOptions::default(),
			)
			.await;
			assert_eq!(response.status(), 200);

			let response = create_attribute(
				&mut app,
				class_audiofile_id,
				"audio_hash",
				AttrDataStub::Hash {
					hash_type: HashType::SHA256,
				},
				AttributeOptions::default(),
			)
			.await;
			assert_eq!(response.status(), 200);

			let response = create_attribute(
				&mut app,
				class_audiofile_id,
				"cover_art",
				AttrDataStub::Reference {
					class: class_covers_id,
				},
				AttributeOptions::default(),
			)
			.await;
			assert_eq!(response.status(), 200);
		}

		// These should fail, repeated name
		{
			let response = create_attribute(
				&mut app,
				class_covers_id,
				"content_hash",
				AttrDataStub::Blob,
				AttributeOptions::default(),
			)
			.await;
			assert_eq!(response.status(), 400);
			assert_eq!(
				response_json::<String>(response).await,
				"an attribute with this name already exists"
			);

			let response = create_attribute(
				&mut app,
				class_audiofile_id,
				"ISRC",
				AttrDataStub::Text,
				AttributeOptions::default(),
			)
			.await;
			assert_eq!(response.status(), 400);
			assert_eq!(
				response_json::<String>(response).await,
				"an attribute with this name already exists"
			);

			let response = rename_attribute(&mut app, attr_title_id, "ISRC").await;
			assert_eq!(response.status(), 400);
			assert_eq!(
				response_json::<String>(response).await,
				"an attribute with this name already exists"
			);
		}

		//
		// MARK: Create items
		//

		{
			let mut transaction = Transaction::new();
			transaction.add_action(TransactionAction::AddItem {
				to_class: class_audiofile_id,
				attributes: vec![(
					attr_title_id,
					AttrData::Text {
						value: "title!".into(),
					}
					.into(),
				)],
			});

			let response = apply_transaction(&mut app, transaction).await;
			assert_eq!(response.status(), 200);
		}
	}
}
