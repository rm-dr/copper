use axum::{
	routing::{delete, post},
	Router,
};
use serde::{Deserialize, Serialize};
use utoipa::{OpenApi, ToSchema};

use super::{class::ClassSelect, RouterState};

mod add;
mod del;

use add::*;
use del::*;

#[derive(Deserialize, Serialize, ToSchema, Debug)]
pub(in crate::api) struct AttrSelect {
	#[serde(flatten)]
	pub class: ClassSelect,
	pub attr: String,
}

#[derive(OpenApi)]
#[openapi(
	tags(),
	paths(del_attr, add_attr),
	components(schemas(AttrSelect, NewClassAttrParams))
)]
pub(super) struct AttrApi;

pub(super) fn router() -> Router<RouterState> {
	Router::new()
		.route("/add", post(add_attr))
		.route("/del", delete(del_attr))
}
