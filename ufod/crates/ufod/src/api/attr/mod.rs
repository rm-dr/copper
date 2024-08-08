use axum::{
	routing::{delete, get, post},
	Router,
};
use utoipa::OpenApi;

use super::RouterState;

mod add;
mod del;
mod find;

use add::*;
use del::*;
use find::*;

#[derive(OpenApi)]
#[openapi(
	tags(),
	paths(del_attr, add_attr, find_attr),
	components(schemas(NewAttrParams, DelAttrRequest, FindAttrRequest))
)]
pub(super) struct AttrApi;

pub(super) fn router() -> Router<RouterState> {
	Router::new()
		.route("/add", post(add_attr))
		.route("/find", get(find_attr))
		.route("/del", delete(del_attr))
}
