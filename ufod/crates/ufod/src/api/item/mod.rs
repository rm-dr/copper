use super::RouterState;
use axum::{routing::get, Router};
use utoipa::OpenApi;

mod attr;
mod list;

use attr::*;
use list::*;

#[derive(OpenApi)]
#[openapi(
	tags(),
	paths(list_item, get_item_attr),
	components(schemas(ItemListRequest, ItemListItem, ItemListData, ItemListResponse))
)]
pub(super) struct ItemApi;

pub(super) fn router() -> Router<RouterState> {
	Router::new()
		.route("/list", get(list_item))
		.route("/attr", get(get_item_attr))
}
