use crate::RouterState;
use axum::{routing::post, Router};
use copper_database::api::{
	client::DatabaseClient,
	transaction::{Transaction, TransactionAction},
};
use utoipa::OpenApi;

mod exec;

use exec::*;

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
