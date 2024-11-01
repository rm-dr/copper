use axum::extract::connect_info::Connected;
use axum::routing::post;
use axum::serve::IncomingStream;
use axum::{extract::DefaultBodyLimit, Router};
use copper_edged::UserInfo;
use copper_itemdb::client::base::client::ItemdbClient;
use copper_itemdb::{AttrDataStub, AttributeInfo, AttributeOptions, ClassInfo, DatasetInfo};
use copper_jobqueue::base::client::JobQueueClient;
use copper_jobqueue::info::QueuedJobCounts;
use copper_util::s3client::S3Client;
use copper_util::HashType;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::auth::AuthHelper;
use crate::config::EdgedConfig;
use crate::database::base::client::DatabaseClient;
use crate::uploader::Uploader;

mod attribute;
mod class;
mod dataset;
mod item;
mod job;
mod login;
mod logout;
mod pipeline;
mod storage;
mod user;

use login::*;
use logout::*;

#[derive(Clone, Debug)]
pub struct CopperConnectInfo {
	pub addr: Arc<SocketAddr>,
}

impl Connected<IncomingStream<'_>> for CopperConnectInfo {
	fn connect_info(target: IncomingStream<'_>) -> Self {
		let addr = target.remote_addr();

		Self {
			addr: Arc::new(addr),
		}
	}
}

pub struct RouterState<Client: DatabaseClient, Itemdb: ItemdbClient> {
	pub config: Arc<EdgedConfig>,
	pub db_client: Arc<Client>,
	pub itemdb_client: Arc<Itemdb>,
	pub jobqueue_client: Arc<dyn JobQueueClient>,
	pub auth: Arc<AuthHelper<Client>>,
	pub s3_client: Arc<S3Client>,
	pub uploader: Arc<Uploader>,
}

// We need to impl this manually, since `DatabaseClient`
// doesn't implement `Clone`
impl<Client: DatabaseClient, Itemdb: ItemdbClient> Clone for RouterState<Client, Itemdb> {
	fn clone(&self) -> Self {
		Self {
			config: self.config.clone(),
			db_client: self.db_client.clone(),
			auth: self.auth.clone(),
			itemdb_client: self.itemdb_client.clone(),
			jobqueue_client: self.jobqueue_client.clone(),
			s3_client: self.s3_client.clone(),
			uploader: self.uploader.clone(),
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
		(path = "/pipeline", api = pipeline::PipelineApi),
		(path = "/storage", api = storage::StorageApi),
		(path = "/job", api = job::JobApi),
		(path = "/item", api = item::ItemApi),
	),
	tags(
		(name = "Copper", description = "Copper edge daemon")
	),
	paths(try_login, logout),
	components(schemas(UserInfo, LoginRequest, AttrDataStub, AttributeOptions, DatasetInfo, AttributeInfo, HashType, ClassInfo, QueuedJobCounts))
)]
struct ApiDoc;

pub(super) fn router<Client: DatabaseClient + 'static, Itemdb: ItemdbClient + 'static>(
	state: RouterState<Client, Itemdb>,
) -> Router {
	Router::new()
		.merge(SwaggerUi::new("/docs").url("/docs/openapi.json", ApiDoc::openapi()))
		//
		.nest("/user", user::router())
		.nest("/dataset", dataset::router())
		.nest("/class", class::router())
		.nest("/attribute", attribute::router())
		.nest("/pipeline", pipeline::router())
		.nest("/storage", storage::router())
		.nest("/job", job::router())
		.nest("/item", item::router())
		//
		.route("/login", post(try_login))
		.route("/logout", post(logout))
		//
		.layer(TraceLayer::new_for_http())
		.layer(DefaultBodyLimit::max(state.config.edged_request_body_limit))
		.with_state(state)
}
