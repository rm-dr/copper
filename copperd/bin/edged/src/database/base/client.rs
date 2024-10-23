//! The database client api

use async_trait::async_trait;
use copper_edged::{PipelineId, PipelineInfo, UserInfo, UserPassword};
use copper_pipelined::json::PipelineJson;
use copper_itemdb::UserId;

use super::errors::{
	pipeline::{
		AddPipelineError, DeletePipelineError, GetPipelineError, ListPipelineError,
		UpdatePipelineError,
	},
	user::{AddUserError, DeleteUserError, GetUserError, UpdateUserError},
};

/// A generic database client
#[async_trait]
pub trait DatabaseClient
where
	Self: Send + Sync,
{
	//
	// MARK: Users
	//

	/// Create a new user
	async fn add_user(
		&self,
		email: &str,
		name: &str,
		password: &UserPassword,
	) -> Result<UserId, AddUserError>;

	/// Get a user by id
	async fn get_user(&self, user: UserId) -> Result<Option<UserInfo>, GetUserError>;

	/// Get a user by email
	async fn get_user_by_email(&self, email: &str) -> Result<Option<UserInfo>, GetUserError>;

	/// Update a user
	async fn update_user(&self, new_info: &UserInfo) -> Result<(), UpdateUserError>;

	/// Delete a user
	async fn del_user(&self, user: UserId) -> Result<(), DeleteUserError>;

	//
	// MARK: Pipelines
	//

	/// Get all a user's pipelines
	async fn list_pipelines(
		&self,
		for_user: UserId,
	) -> Result<Vec<PipelineInfo>, ListPipelineError>;

	/// Create a new pipeline
	async fn add_pipeline(
		&self,
		for_user: UserId,
		name: &str,
		pipeline: &PipelineJson,
	) -> Result<PipelineInfo, AddPipelineError>;

	/// Get a pipeline by id
	async fn get_pipeline(
		&self,
		pipeline: PipelineId,
	) -> Result<Option<PipelineInfo>, GetPipelineError>;

	/// Update a pipeline
	async fn update_pipeline(
		&self,
		new_info: &PipelineInfo,
	) -> Result<PipelineInfo, UpdatePipelineError>;

	/// Delete a pipeline
	async fn del_pipeline(&self, pipeline: PipelineId) -> Result<(), DeletePipelineError>;
}
