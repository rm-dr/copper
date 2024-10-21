use crate::database::base::{client::DatabaseClient, errors::transaction::ApplyTransactionError};
use axum::{
	extract::{OriginalUri, State},
	http::{HeaderMap, StatusCode},
	response::{IntoResponse, Response},
	Json,
};
use copper_storaged::{ApplyTransactionApiError, Transaction};
use serde::Deserialize;
use tracing::error;
use utoipa::ToSchema;

use crate::api::RouterState;

#[derive(Deserialize, ToSchema, Debug)]
pub(super) struct ExecTransactionRequest {
	transaction: Transaction,
}

/// Apply a transaction
#[utoipa::path(
	post,
	path = "/apply",
	responses(
		(status = 200, description = "Transaction executed successfully"),
		(status = 400, description = "Bad request", body = String),
		(status = 500, description = "Transaction failed"),
	),
	security(
		("bearer" = []),
	)
)]
pub(super) async fn apply_transaction<Client: DatabaseClient>(
	headers: HeaderMap,
	OriginalUri(uri): OriginalUri,
	State(state): State<RouterState<Client>>,
	Json(payload): Json<ExecTransactionRequest>,
) -> Response {
	if !state.config.header_has_valid_auth(&uri, &headers) {
		return StatusCode::UNAUTHORIZED.into_response();
	};

	let res = state.client.apply_transaction(payload.transaction).await;

	return match res {
		Ok(()) => StatusCode::OK.into_response(),

		Err(ApplyTransactionError::DbError(e)) => {
			error!(
				message = "Database error while running transaction",
				error = ?e
			);
			StatusCode::INTERNAL_SERVER_ERROR.into_response()
		}

		Err(ApplyTransactionError::ReferencedBadAction) => (
			StatusCode::BAD_REQUEST,
			Json(ApplyTransactionApiError::ReferencedBadAction),
		)
			.into_response(),

		Err(ApplyTransactionError::ReferencedNoneResult) => (
			StatusCode::BAD_REQUEST,
			Json(ApplyTransactionApiError::ReferencedNoneResult),
		)
			.into_response(),

		Err(ApplyTransactionError::ReferencedResultWithBadType) => (
			StatusCode::BAD_REQUEST,
			Json(ApplyTransactionApiError::ReferencedResultWithBadType),
		)
			.into_response(),

		Err(ApplyTransactionError::AddItemError(error)) => (
			StatusCode::BAD_REQUEST,
			Json(ApplyTransactionApiError::AddItem { error }),
		)
			.into_response(),
	};
}
