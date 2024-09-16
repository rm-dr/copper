//! The database client api

use async_trait::async_trait;
use copper_edged::{UserId, UserInfo, UserPassword};

use super::errors::user::{
	AddUserError, DeleteUserError, GetUserByEmailError, GetUserError, UpdateUserError,
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

	/// Create a new dataset
	async fn add_user(
		&self,
		email: &str,
		name: &str,
		password: &UserPassword,
	) -> Result<UserId, AddUserError>;

	/// Get a user by id
	async fn get_user(&self, user: UserId) -> Result<UserInfo, GetUserError>;

	/// Get a user by email
	async fn get_user_by_email(&self, email: &str) -> Result<UserInfo, GetUserByEmailError>;

	/// Update a user
	async fn update_user(&self, new_info: &UserInfo) -> Result<(), UpdateUserError>;

	/// Delete a user
	async fn del_user(&self, user: UserId) -> Result<(), DeleteUserError>;
}
