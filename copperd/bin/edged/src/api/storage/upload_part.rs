use axum::{
	body::Bytes,
	extract::{Multipart, Path, State},
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use axum_extra::extract::CookieJar;
use copper_itemdb::client::base::client::ItemdbClient;
use tracing::{error, warn};

use crate::{api::RouterState, uploader::errors::UploadFragmentError};
use crate::{database::base::client::DatabaseClient, uploader::UploadJobId};

/// Upload a part of a file.
/// TODO: enforce 5MB minimum size
#[utoipa::path(
	post,
	path = "/upload/{upload_id}/part",
	params(
		("upload_id", description = "Upload id"),
	),
	responses(
		(status = 200, description = "Part uploaded successfully"),
		(status = 400, description = "Invalid request", body = String),
		(status = 401, description = "Unauthorized", body = String),
		(status = 404, description = "Upload job not found", body = String),
		(status = 500, description = "Internal server error"),
	)
)]
pub(super) async fn upload_part<Client: DatabaseClient, Itemdb: ItemdbClient>(
	jar: CookieJar,
	State(state): State<RouterState<Client, Itemdb>>,
	Path(job_id): Path<UploadJobId>,
	mut multipart: Multipart,
) -> Response {
	let user = match state.auth.auth_or_logout(&state, &jar).await {
		Err(x) => return x,
		Ok(user) => user,
	};

	// Parse multipart data
	let mut data: Option<Bytes> = None;
	while let Some(field) = multipart.next_field().await.unwrap() {
		let name = field.name().unwrap().to_string();

		match &name[..] {
			"part_data" => {
				if data.is_some() {
					warn!(
						message = "Multiple `part_data` fields in upload request",
						?job_id,
					);

					return (
						StatusCode::BAD_REQUEST,
						Json("Multiple `part_data` fields in upload request"),
					)
						.into_response();
				}

				data = match field.bytes().await {
					Ok(x) => Some(x),
					Err(error) => {
						warn!(
							message = "Failed reading part data, client probably disconnected",
							?job_id,
							?error
						);

						return (
							StatusCode::INTERNAL_SERVER_ERROR,
							Json("Failed reading part data, client probably disconnected"),
						)
							.into_response();
					}
				};
			}

			_ => {
				warn!(message = "Bad field name in upload request", ?job_id);

				return (
					StatusCode::BAD_REQUEST,
					Json(format!("unexpected field `{name}`")),
				)
					.into_response();
			}
		}
	}

	if data.is_none() {
		warn!(
			message = "part_data field was missing in upload request",
			?job_id,
		);

		return (StatusCode::BAD_REQUEST, Json("Missing part_data field")).into_response();
	}

	return match state
		.uploader
		.upload_part(user.id, &job_id, &data.unwrap(), None)
		.await
	{
		Ok(()) => StatusCode::OK.into_response(),

		Err(UploadFragmentError::NotMyUpload) | Err(UploadFragmentError::BadUpload) => {
			return StatusCode::NOT_FOUND.into_response();
		}

		Err(UploadFragmentError::S3Error(error)) => {
			error!(message = "S3 error while uploading part", ?error);
			return StatusCode::INTERNAL_SERVER_ERROR.into_response();
		}
	};
}
