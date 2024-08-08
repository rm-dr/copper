use axum::{
	routing::{delete, get, post},
	Router,
};
use list::list_classes;
use serde::{Deserialize, Serialize};
use utoipa::{OpenApi, ToSchema};

use super::RouterState;

mod add;
mod del;
mod list;

use add::*;
use del::*;
use list::*;

#[derive(Deserialize, Serialize, ToSchema, Debug)]
pub(in crate::api) struct ClassSelect {
	pub dataset: String,
	pub class: String,
}

#[derive(OpenApi)]
#[openapi(
	tags(),
	paths(list_classes, add_class, del_class),
	components(schemas(ClassInfoRequest, ClassInfo, AttrInfo, ClassSelect))
)]
pub(super) struct ClassApi;

pub(super) fn router() -> Router<RouterState> {
	Router::new()
		.route("/list", get(list_classes))
		.route("/add", post(add_class))
		.route("/del", delete(del_class))
}
