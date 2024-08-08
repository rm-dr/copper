use super::RouterState;
use axum::{
	routing::{delete, get, post},
	Router,
};
use utoipa::OpenApi;

mod addgroup;
mod adduser;
mod delgroup;
mod deluser;
mod listgroup;
mod login;
mod logout;
mod me;

use addgroup::*;
use adduser::*;
use delgroup::*;
use deluser::*;
use listgroup::*;
use login::*;
use logout::*;
use me::*;

#[derive(OpenApi)]
#[openapi(
	tags(),
	paths(
		try_login, add_user, add_group, del_group, del_user, logout, list_group, get_me
	),
	components(schemas(
		LoginRequest,
		AdduserRequest,
		AddgroupRequest,
		DeluserRequest,
		DelgroupRequest,
		ListgroupInfo
	))
)]
pub(super) struct AuthApi;

pub(super) fn router() -> Router<RouterState> {
	Router::new()
		.route("/login", post(try_login))
		.route("/logout", post(logout))
		.route("/user/add", post(add_user))
		.route("/me", get(get_me))
		.route("/user/del", delete(del_user))
		.route("/group/add", post(add_group))
		.route("/group/del", delete(del_group))
		.route("/group/list", get(list_group))
}
