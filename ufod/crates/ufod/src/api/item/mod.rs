use super::RouterState;
use axum::{routing::get, Router};
use utoipa::OpenApi;

mod list;

use list::*;

#[derive(OpenApi)]
#[openapi(
	tags(),
	paths(list_item),
	components(schemas(ItemListRequest, ItemListItem, ItemListData, ItemListResponse))
)]
pub(super) struct ItemApi;

pub(super) fn router() -> Router<RouterState> {
	Router::new().route("/list", get(list_item))
}
