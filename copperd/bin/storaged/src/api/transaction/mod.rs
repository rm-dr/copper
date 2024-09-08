use crate::RouterState;
use axum::{routing::post, Router};
use storaged_database::api::{
	client::DatabaseClient,
	transaction::{Transaction, TransactionAction},
};
use utoipa::OpenApi;

mod apply;

use apply::*;

#[derive(OpenApi)]
#[openapi(
	tags(),
	paths(apply_transaction),
	components(schemas(Transaction, TransactionAction, ExecTransactionRequest))
)]
pub(super) struct TransactionApi;

pub(super) fn router<Client: DatabaseClient + 'static>() -> Router<RouterState<Client>> {
	Router::new().route("/apply", post(apply_transaction))
}
