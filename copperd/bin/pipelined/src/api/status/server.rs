use axum::{
	extract::{OriginalUri, State},
	http::{HeaderMap, StatusCode},
	response::{IntoResponse, Response},
	Json,
};
use serde::{Deserialize, Serialize};
use smartstring::{LazyCompact, SmartString};
use utoipa::ToSchema;

use crate::RouterState;

/// The server's status
#[derive(Deserialize, Serialize, ToSchema, Debug)]
pub(super) struct ServerStatus {
	/// This server's version
	#[schema(
		example = json!(env!("CARGO_PKG_VERSION")),
		value_type = String,
	)]
	pub version: SmartString<LazyCompact>,

	/// The maximum request size this server supports, in bytes
	#[schema(example = 2_000_000)]
	pub request_body_limit: usize,
}

/// Get server status
#[utoipa::path(
	get,
	path = "",
	responses(
		(status = 200, description = "Server status", body = ServerStatus),
		(status = 401, description = "Unauthorized")
	)
)]
pub(super) async fn get_server_status(
	headers: HeaderMap,
	OriginalUri(uri): OriginalUri,
	State(state): State<RouterState>,
) -> Response {
	if !state.config.header_has_valid_auth(&uri, &headers) {
		return StatusCode::UNAUTHORIZED.into_response();
	};

	return (
		StatusCode::OK,
		Json(ServerStatus {
			version: env!("CARGO_PKG_VERSION").into(),
			request_body_limit: state.config.pipelined_request_body_limit,
		}),
	)
		.into_response();
}
