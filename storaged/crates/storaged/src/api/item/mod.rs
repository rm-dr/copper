use super::RouterState;
use axum::{routing::get, Router};
use utoipa::OpenApi;

mod attr;
mod get;
mod list;

use attr::*;
use get::*;
use list::*;

#[derive(OpenApi)]
#[openapi(
	tags(),
	paths(get_item, list_item, get_item_attr),
	components(schemas(
		ItemListRequest,
		ItemListItem,
		ItemListData,
		ItemListResponse,
		ItemGetRequest
	))
)]
pub(super) struct ItemApi;

pub(super) fn router() -> Router<RouterState> {
	Router::new()
		.route("/list", get(list_item))
		.route("/get", get(get_item))
		.route("/attr", get(get_item_attr))
}
