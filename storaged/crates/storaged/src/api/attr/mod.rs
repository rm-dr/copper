use axum::{
	routing::{delete, get, post},
	Router,
};
use utoipa::OpenApi;

use super::RouterState;

mod add;
mod del;
mod find;
mod get;
mod rename;

use add::*;
use del::*;
use find::*;
use get::*;
use rename::*;

#[derive(OpenApi)]
#[openapi(
	tags(),
	paths(del_attr, add_attr, find_attr, get_attr, rename_attr),
	components(schemas(
		NewAttrParams,
		DelAttrRequest,
		FindAttrRequest,
		GetAttrRequest,
		RenameAttrRequest
	))
)]
pub(super) struct AttrApi;

pub(super) fn router() -> Router<RouterState> {
	Router::new()
		.route("/add", post(add_attr))
		.route("/rename", post(rename_attr))
		.route("/find", get(find_attr))
		.route("/get", get(get_attr))
		.route("/del", delete(del_attr))
}
