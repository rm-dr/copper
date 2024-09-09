use axum::{routing::post, Router};
use copper_storaged::{Transaction, TransactionAction};
use utoipa::OpenApi;

use crate::database::base::client::DatabaseClient;

mod apply;

use apply::*;

use super::RouterState;

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
