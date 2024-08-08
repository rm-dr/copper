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

use crate::{
	api::RouterState,
	helpers::maindb::auth::{GroupInfo, UserInfo},
};

#[derive(Debug, Serialize, ToSchema)]
pub(super) struct ListgroupInfo {
	group_info: GroupInfo,
	users: Vec<UserInfo>,
}

/// List all groups
#[utoipa::path(
	get,
	path = "/group/list",
	responses(
		(status = 200, description = "List of groups", body = Vec<ListgroupInfo>),
		(status = 400, description = "Could not list groups"),
		(status = 500, description = "Internal server error", body = String),
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
			let mut out = Vec::new();
			for g in l {
				let users = match state.main_db.auth.list_users(g.id).await {
					Ok(x) => x,
					Err(e) => {
						error!(
							message = "Could not list users",
							group = ?g,
							error = ?e
						);
						return (StatusCode::INTERNAL_SERVER_ERROR, "Could not list users")
							.into_response();
					}
				};

				out.push(ListgroupInfo {
					users,
					group_info: g,
				})
			}

			return Json(out).into_response();
		}

		Err(e) => {
			error!(
				message = "Could not list groups",
				error = ?e
			);
			return (StatusCode::INTERNAL_SERVER_ERROR, "Could not list groups").into_response();
		}
	};
}
