use axum::{
	extract::State,
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use axum_extra::extract::CookieJar;
use serde::Serialize;
use tracing::error;
use utoipa::ToSchema;

use crate::{api::RouterState, helpers::maindb::auth::GroupId};

#[derive(Debug, Serialize, ToSchema)]
pub(super) struct ListgroupResponse {
	groups: Vec<ListgroupInfo>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(super) struct ListgroupInfo {
	name: String,
	id: GroupId,
	parent: Option<GroupId>,
}

/// Create a new group
#[utoipa::path(
	get,
	path = "/group/list",
	responses(
		(status = 200, description = "List of groups", body=ListgroupResponse),
		(status = 400, description = "Could not create group"),
		(status = 500, description = "Internal server error", body=String),
		(status = 401, description = "Unauthorized")
	)
)]
pub(super) async fn list_group(jar: CookieJar, State(state): State<RouterState>) -> Response {
	let user_info = match state.main_db.auth.auth_or_logout(&jar).await {
		Err(x) => return x,
		Ok(u) => u,
	};

	match state.main_db.auth.list_groups(user_info.group.id).await {
		Ok(l) => {
			return Json(ListgroupResponse {
				groups: l
					.into_iter()
					.map(|x| ListgroupInfo {
						name: x.name.into(),
						id: x.id,
						parent: x.parent,
					})
					.collect(),
			})
			.into_response();
		}

		Err(e) => {
			error!(
				message = "Could not list groups",
				error = ?e
			);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				format!("Could not list groups"),
			)
				.into_response();
		}
	};
}
