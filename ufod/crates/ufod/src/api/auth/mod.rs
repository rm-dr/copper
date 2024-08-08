use super::RouterState;
use axum::{
	routing::{delete, post},
	Router,
};
use utoipa::OpenApi;

mod addgroup;
mod adduser;
mod delgroup;
mod deluser;
mod login;

use addgroup::*;
use adduser::*;
use delgroup::*;
use deluser::*;
use login::*;

#[derive(OpenApi)]
#[openapi(
	tags(),
	paths(try_login, add_user, add_group, del_group, del_user),
	components(schemas(
		LoginRequest,
		AdduserRequest,
		AddgroupRequest,
		DeluserRequest,
		DelgroupRequest
	))
)]
pub(super) struct AuthApi;

pub(super) fn router() -> Router<RouterState> {
	Router::new()
		.route("/login", post(try_login))
		.route("/user", post(add_user))
		.route("/user", delete(del_user))
		.route("/group", post(add_group))
		.route("/group", delete(del_group))
}
