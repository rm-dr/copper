use axum::{extract::DefaultBodyLimit, Router};
use copper_ds_core::{
	api::{
		blob::BlobHandle,
		meta::{AttrInfo, AttributeOptions, ClassInfo},
	},
	data::{HashType, MetastoreDataStub},
};
use copper_ds_impl::DatasetType;
use copper_node_base::{
	data::{CopperData, CopperDataStub},
	CopperContext,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use copper_pipeline::runner::runner::PipelineRunner;

mod attr;
mod auth;
mod class;
mod dataset;
mod item;
mod pipeline;
mod status;
mod upload;

use crate::{
	config::CopperConfig,
	maindb::{
		auth::{GroupId, GroupInfo, UserInfo},
		MainDB,
	},
	uploader::Uploader,
};

#[derive(Clone)]
pub struct RouterState {
	pub config: Arc<CopperConfig>,
	pub runner: Arc<Mutex<PipelineRunner<CopperData, CopperContext>>>,
	pub main_db: Arc<MainDB>,
	pub uploader: Arc<Uploader>,
}

#[derive(OpenApi)]
#[openapi(
	//modifiers(&BearerSecurityAddon),
	nest(
		(path = "/status", api = status::StatusApi),
		(path = "/upload", api = upload::UploadApi),
		(path = "/dataset", api = dataset::DatasetApi),
		(path = "/pipeline", api = pipeline::PipelineApi),
		(path = "/class", api = class::ClassApi),
		(path = "/attr", api = attr::AttrApi),
		(path = "/item", api = item::ItemApi),
		(path = "/auth", api = auth::AuthApi)
	),
	tags(
		(name = "Copper", description = "Copper backend daemon")
	),
	// All schema structs defined outside `crate::api` go here
	components(schemas(
		MetastoreDataStub,
		HashType,
		AttributeOptions,
		DatasetType,
		BlobHandle,
		GroupId,
		GroupInfo,
		UserInfo,
		AttrInfo,
		ClassInfo,
		CopperDataStub,
		CopperData,
	))
)]
struct ApiDoc;

pub(super) fn router(state: RouterState) -> Router {
	Router::new()
		.merge(SwaggerUi::new("/docs").url("/docs/openapi.json", ApiDoc::openapi()))
		//
		.nest("/upload", upload::router())
		.nest("/status", status::router())
		.nest("/dataset", dataset::router())
		.nest("/pipeline", pipeline::router())
		.nest("/class", class::router())
		.nest("/attr", attr::router())
		.nest("/item", item::router())
		.nest("/auth", auth::router())
		//
		.layer(TraceLayer::new_for_http())
		.layer(DefaultBodyLimit::max(
			state.config.network.request_body_limit,
		))
		.with_state(state)
}
