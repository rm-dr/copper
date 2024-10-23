use crate::database::base::client::DatabaseClient;
use crate::RouterState;
use axum::{routing::post, Router};
use copper_storage::database::base::client::StorageDatabaseClient;
use utoipa::OpenApi;

mod finish_upload;
mod start_upload;
mod upload_part;

use finish_upload::*;
use start_upload::*;
use upload_part::*;

#[derive(OpenApi)]
#[openapi(
	tags(),
	paths(start_upload, upload_part, finish_upload),
	components(schemas(StartUploadRequest, StartUploadResponse))
)]
pub(super) struct StorageApi;

pub(super) fn router<
	Client: DatabaseClient + 'static,
	StorageClient: StorageDatabaseClient + 'static,
>() -> Router<RouterState<Client, StorageClient>> {
	Router::new()
		.route("/upload", post(start_upload))
		.route("/upload/:upload_id/part", post(upload_part))
		.route("/upload/:upload_id/finish", post(finish_upload))
}
