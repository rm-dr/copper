use axum::{
	extract::State,
	http::StatusCode,
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
	),
)]
pub(super) async fn get_server_status(State(state): State<RouterState>) -> Response {
	return (
		StatusCode::OK,
		Json(ServerStatus {
			version: env!("CARGO_PKG_VERSION").into(),
			request_body_limit: state.config.request_body_limit,
		}),
	)
		.into_response();
}
