use axum::routing::post;
use axum::{extract::DefaultBodyLimit, Router};
use copper_edged::UserInfo;
use copper_storaged::client::StoragedClient;
use copper_storaged::{AttrDataStub, AttributeInfo, AttributeOptions, ClassInfo, DatasetInfo};
use copper_util::HashType;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::auth::AuthHelper;
use crate::database::base::client::DatabaseClient;

use crate::config::EdgedConfig;

mod attribute;
mod class;
mod dataset;
mod login;
mod logout;
mod user;

use login::*;
use logout::*;

pub struct RouterState<Client: DatabaseClient> {
	pub config: Arc<EdgedConfig>,
	pub db_client: Arc<Client>,
	pub storaged_client: Arc<dyn StoragedClient>,
	pub auth: Arc<AuthHelper<Client>>,
}

// We need to impl this manually, since `DatabaseClient`
// doesn't implement `Clone`
impl<Client: DatabaseClient> Clone for RouterState<Client> {
	fn clone(&self) -> Self {
		Self {
			config: self.config.clone(),
			db_client: self.db_client.clone(),
			auth: self.auth.clone(),
			storaged_client: self.storaged_client.clone(),
		}
	}
}

#[allow(non_camel_case_types)]
#[derive(OpenApi)]
#[openapi(
	nest(
		(path = "/user", api = user::UserApi),
		(path = "/dataset", api = dataset::DatasetApi),
		(path = "/class", api = class::ClassApi),
		(path = "/attribute", api = attribute::AttributeApi),
	),
	tags(
		(name = "Copper", description = "Copper edge daemon")
	),
	paths(try_login, logout),
	components(schemas(UserInfo, LoginRequest, AttrDataStub, AttributeOptions, DatasetInfo, AttributeInfo, HashType, ClassInfo))
)]
struct ApiDoc;

pub(super) fn router<Client: DatabaseClient + 'static>(state: RouterState<Client>) -> Router {
	Router::new()
		.merge(SwaggerUi::new("/docs").url("/docs/openapi.json", ApiDoc::openapi()))
		//
		.nest("/user", user::router())
		.nest("/dataset", dataset::router())
		.nest("/class", class::router())
		.nest("/attribute", attribute::router())
		//
		.route("/login", post(try_login))
		.route("/logout", post(logout))
		//
		.layer(TraceLayer::new_for_http())
		.layer(DefaultBodyLimit::max(state.config.edged_request_body_limit))
		.with_state(state)
}
