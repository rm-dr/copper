use axum::{extract::DefaultBodyLimit, Router};
use copper_database::api::{
	data::{HashType, MetastoreDataStub},
	AttrInfo, AttributeOptions, ClassInfo,
};
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::config::StoragedConfig;

// mod dataset;
mod status;
// mod attr;
// mod class;
// mod item;

#[derive(Clone)]
pub struct RouterState {
	pub config: Arc<StoragedConfig>,
}

#[derive(OpenApi)]
#[openapi(
	//modifiers(&BearerSecurityAddon),
	nest(
		(path = "/status", api = status::StatusApi),
		// (path = "/dataset", api = dataset::DatasetApi),
		// (path = "/class", api = class::ClassApi),
		// (path = "/attr", api = attr::AttrApi),
		// (path = "/item", api = item::ItemApi),
	),
	tags(
		(name = "Copper", description = "Copper backend daemon")
	),
	// All schema structs defined outside `crate::api` go here
	components(schemas(
		MetastoreDataStub,
		HashType,
		AttributeOptions,
		AttrInfo,
		ClassInfo,
	))
)]
struct ApiDoc;

pub(super) fn router(state: RouterState) -> Router {
	Router::new()
		.merge(SwaggerUi::new("/docs").url("/docs/openapi.json", ApiDoc::openapi()))
		//
		.nest("/status", status::router())
		// .nest("/dataset", dataset::router())
		// .nest("/class", class::router())
		// .nest("/attr", attr::router())
		// .nest("/item", item::router())
		//
		.layer(TraceLayer::new_for_http())
		.layer(DefaultBodyLimit::max(state.config.request_body_limit))
		.with_state(state)
}
