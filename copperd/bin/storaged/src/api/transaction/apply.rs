use crate::database::base::{client::DatabaseClient, errors::transaction::ApplyTransactionError};
use axum::{
	extract::State,
	http::{HeaderMap, StatusCode},
	response::{IntoResponse, Response},
	Json,
};
use copper_storaged::Transaction;
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
	_headers: HeaderMap,
	State(state): State<RouterState<Client>>,
	Json(payload): Json<ExecTransactionRequest>,
) -> Response {
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

		Err(e) => {
			error!(
				message = "error while running transaction",
				error = ?e
			);
			StatusCode::INTERNAL_SERVER_ERROR.into_response()
		}
	};
}
