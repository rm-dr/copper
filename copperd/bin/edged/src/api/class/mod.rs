use crate::database::base::client::DatabaseClient;
use crate::RouterState;
use axum::{
	routing::{delete, get, patch, post},
	Router,
};
use utoipa::OpenApi;

mod add_attribute;
mod del;
mod get;
mod items;
mod rename;

use add_attribute::*;
use del::*;
use get::*;
use items::*;
use rename::*;

#[derive(OpenApi)]
#[openapi(
	tags(),
	paths(rename_class, del_class, get_class, add_attribute, list_items),
	components(schemas(
		RenameClassRequest,
		NewAttributeRequest,
		ItemlistItemInfo,
		ItemAttrData,
		PrimaryAttrData,
		ItemListResponse
	))
)]
pub(super) struct ClassApi;

pub(super) fn router<Client: DatabaseClient + 'static>() -> Router<RouterState<Client>> {
	Router::new()
		.route("/:class_id", get(get_class))
		.route("/:class_id/items", get(list_items))
		.route("/:class_id", delete(del_class))
		.route("/:class_id", patch(rename_class))
		//
		.route("/:class_id/attribute", post(add_attribute))
}
