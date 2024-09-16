use axum::routing::post;
use axum::{extract::DefaultBodyLimit, Router};
use copper_edged::UserInfo;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::auth::AuthHelper;
use crate::database::base::client::DatabaseClient;

use crate::config::EdgedConfig;

mod login;
mod logout;
mod user;

use login::*;
use logout::*;

pub struct RouterState<Client: DatabaseClient> {
	pub config: Arc<EdgedConfig>,
	pub client: Arc<Client>,
	pub auth: Arc<AuthHelper<Client>>,
}

// We need to impl this manually, since `DatabaseClient`
// doesn't implement `Clone`
impl<Client: DatabaseClient> Clone for RouterState<Client> {
	fn clone(&self) -> Self {
		Self {
			config: self.config.clone(),
			client: self.client.clone(),
			auth: self.auth.clone(),
		}
	}
}

#[derive(OpenApi)]
#[openapi(
	nest(
		(path = "/user", api = user::UserApi),
	),
	tags(
		(name = "Copper", description = "Copper edge daemon")
	),
	paths(try_login, logout),
	components(schemas(UserInfo, LoginRequest))
)]
struct ApiDoc;

pub(super) fn router<Client: DatabaseClient + 'static>(state: RouterState<Client>) -> Router {
	Router::new()
		.merge(SwaggerUi::new("/docs").url("/docs/openapi.json", ApiDoc::openapi()))
		//
		.nest("/user", user::router())
		//
		.route("/login", post(try_login))
		.route("/logout", post(logout))
		//
		.layer(TraceLayer::new_for_http())
		.layer(DefaultBodyLimit::max(state.config.edged_request_body_limit))
		.with_state(state)
}
