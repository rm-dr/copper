use axum::{extract::DefaultBodyLimit, Router};
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use utoipa::{
	openapi::security::{Http, HttpAuthScheme, SecurityScheme},
	Modify, OpenApi,
};
use utoipa_swagger_ui::SwaggerUi;

use copper_database::api::{DatabaseClient, DatasetInfo};

use crate::config::StoragedConfig;

mod dataset;
mod status;
// mod attr;
// mod class;
// mod item;

pub struct RouterState<Client: DatabaseClient> {
	pub config: Arc<StoragedConfig>,
	pub client: Arc<Client>,
}

impl<Client: DatabaseClient> Clone for RouterState<Client> {
	fn clone(&self) -> Self {
		Self {
			config: self.config.clone(),
			client: self.client.clone(),
		}
	}
}

struct BearerSecurityAddon;
impl Modify for BearerSecurityAddon {
	fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
		if let Some(components) = openapi.components.as_mut() {
			components.add_security_scheme(
				"bearer",
				SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
			)
		}
	}
}

#[derive(OpenApi)]
#[openapi(
	modifiers(&BearerSecurityAddon),
	nest(
		(path = "/status", api = status::StatusApi),
		(path = "/dataset", api = dataset::DatasetApi),
		// (path = "/class", api = class::ClassApi),
		// (path = "/attr", api = attr::AttrApi),
		// (path = "/item", api = item::ItemApi),
	),
	tags(
		(name = "Copper", description = "Copper backend daemon")
	),
	// All schema structs defined outside `crate::api` go here
	components(schemas(
		DatasetInfo,
	))
)]
struct ApiDoc;

pub(super) fn router<Client: DatabaseClient + 'static>(state: RouterState<Client>) -> Router {
	Router::new()
		.merge(SwaggerUi::new("/docs").url("/docs/openapi.json", ApiDoc::openapi()))
		//
		.nest("/status", status::router())
		.nest("/dataset", dataset::router())
		// .nest("/class", class::router())
		// .nest("/attr", attr::router())
		// .nest("/item", item::router())
		//
		.layer(TraceLayer::new_for_http())
		.layer(DefaultBodyLimit::max(state.config.request_body_limit))
		.with_state(state)
}
