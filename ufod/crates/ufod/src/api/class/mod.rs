use axum::{
	routing::{delete, get, post},
	Router,
};
use list::list_classes;
use utoipa::OpenApi;

use super::RouterState;

mod add;
mod del;
mod list;

use add::*;
use del::*;
use list::*;

#[derive(OpenApi)]
#[openapi(
	tags(),
	paths(list_classes, add_class, del_class),
	components(schemas(ClassListRequest, ExtendedClassInfo, NewClassRequest, DelClassRequest))
)]
pub(super) struct ClassApi;

pub(super) fn router() -> Router<RouterState> {
	Router::new()
		.route("/list", get(list_classes))
		.route("/add", post(add_class))
		.route("/del", delete(del_class))
}
