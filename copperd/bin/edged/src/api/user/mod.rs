use crate::database::base::client::DatabaseClient;
use crate::RouterState;
use axum::{
	routing::{delete, get, patch, post},
	Router,
};
use copper_storage::database::base::client::StorageDatabaseClient;
use utoipa::OpenApi;

mod add;
mod del;
mod me;
mod update;

use add::*;
use del::*;
use me::*;
use update::*;

#[derive(OpenApi)]
#[openapi(
	tags(),
	paths(add_user, update_user, del_user, get_me),
	components(schemas(NewUserRequest, UpdateUserRequest))
)]
pub(super) struct UserApi;

pub(super) fn router<
	Client: DatabaseClient + 'static,
	StorageClient: StorageDatabaseClient + 'static,
>() -> Router<RouterState<Client, StorageClient>> {
	Router::new()
		.route("/", post(add_user))
		.route("/me", get(get_me))
		.route("/:user_id", delete(del_user))
		.route("/:user_id", patch(update_user))
}
