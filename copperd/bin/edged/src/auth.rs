use std::{error::Error, fmt::Display, marker::PhantomData};

use axum::{
	http::{header::SET_COOKIE, StatusCode},
	response::{AppendHeaders, IntoResponse, Response},
	Json,
};
use axum_extra::extract::{
	cookie::{Cookie, Expiration, SameSite},
	CookieJar,
};
use copper_edged::{UserId, UserInfo};
use rand::{distributions::Alphanumeric, Rng};
use smartstring::{LazyCompact, SmartString};
use time::{Duration, OffsetDateTime};
use tokio::sync::Mutex;
use tracing::error;

use crate::database::base::{
	client::DatabaseClient,
	errors::user::{GetUserByEmailError, GetUserError},
};

use super::RouterState;

pub const AUTH_COOKIE_NAME: &str = "copper_auth";
const AUTH_TOKEN_LENGTH: usize = 32;
const AUTH_TOKEN_LIFE_HOURS: i64 = 24;

//
// MARK: Errors
//

/// An error we can encounter when getting user info
#[derive(Debug)]
pub enum LoginError {
	/// Database error
	DbError(Box<dyn Error + Send + Sync>),
}

impl Display for LoginError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::DbError(_) => write!(f, "database backend error"),
		}
	}
}

impl Error for LoginError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::DbError(x) => Some(x.as_ref()),
		}
	}
}

//
// MARK: Helpers
//

#[derive(Debug, Clone)]
pub struct AuthToken {
	pub user: UserId,
	pub token: SmartString<LazyCompact>,
	pub expires: OffsetDateTime,
}

impl AuthToken {
	async fn new(active_tokens: &[Self], user: UserId) -> Self {
		let token = 'outer: loop {
			let rand: String = rand::thread_rng()
				.sample_iter(&Alphanumeric)
				.take(AUTH_TOKEN_LENGTH)
				.map(char::from)
				.collect();
			let token = format!("copper_{}_{rand}", i64::from(user));

			// Make sure token isn't already used
			for t in active_tokens.iter() {
				if t.token == token {
					continue 'outer;
				}
			}
			break token;
		};

		AuthToken {
			user,
			token: token.into(),
			expires: OffsetDateTime::now_utc()
				.checked_add(Duration::hours(AUTH_TOKEN_LIFE_HOURS))
				.unwrap(),
		}
	}
}

//
// MARK: Auth
//

pub struct AuthHelper<Client: DatabaseClient> {
	_p: PhantomData<Client>,
	active_tokens: Mutex<Vec<AuthToken>>,
}

impl<Client: DatabaseClient> AuthHelper<Client> {
	pub fn new() -> Self {
		Self {
			_p: PhantomData {},
			active_tokens: Mutex::new(Vec::new()),
		}
	}

	pub async fn try_login(
		&self,
		state: &RouterState<Client>,
		email: &str,
		password: &str,
	) -> Result<Option<AuthToken>, LoginError> {
		let user = match state.client.get_user_by_email(email).await {
			Ok(user) => user,
			Err(GetUserByEmailError::NotFound) => return Ok(None),
			Err(GetUserByEmailError::DbError(e)) => return Err(LoginError::DbError(e)),
		};

		if user.password.check_password(password) {
			let mut tokens = self.active_tokens.lock().await;
			let t = AuthToken::new(&tokens, user.id).await;
			tokens.push(t.clone());

			return Ok(Some(t));
		}

		return Ok(None);
	}

	/// Look for an authentication cookie in `jar`.
	/// If it is there, return the logged in user's info.
	/// If it isn't (or is invalid), return None.
	pub async fn check_cookies(
		&self,
		state: &RouterState<Client>,
		jar: &CookieJar,
	) -> Result<Option<UserInfo>, GetUserError> {
		let token = if let Some(h) = jar.get(AUTH_COOKIE_NAME) {
			h.value()
		} else {
			return Ok(None);
		};

		for t in self.active_tokens.lock().await.iter() {
			if t.token == token {
				// Expired logins are invalid
				// These will be cleaned up by `auth_or_logout`
				// (if the browser doesn't do so automatically)
				if t.expires < OffsetDateTime::now_utc() {
					return Ok(None);
				}

				return match state.client.get_user(t.user).await {
					Ok(user) => Ok(Some(user)),
					Err(GetUserError::NotFound) => {
						// Tried to authenticate with a user that doesn't exist.
						// This probably happened because our user was deleted.
						// Invalidate this session and return None.
						self.terminate_session(jar).await;
						Ok(None)
					}
					Err(e) => Err(e),
				};
			}
		}

		return Ok(None);
	}

	/// Invalidate any auth cookies in the given jar.
	/// Returns one of the tokens that was invalidated.
	pub async fn terminate_session(&self, jar: &CookieJar) -> Option<AuthToken> {
		let token = if let Some(h) = jar.get(AUTH_COOKIE_NAME) {
			h.value()
		} else {
			return None;
		};

		let mut active_tokens = self.active_tokens.lock().await;
		let mut i = 0;
		let mut x = None;
		while i < active_tokens.len() {
			if active_tokens[i].token == token {
				x = Some(active_tokens.swap_remove(i));
			} else {
				i += 1;
			}
		}

		return x;
	}

	/// Match a user to an authentication token or log out.
	/// This is a convenient wrapper around `self.check_cookies`
	pub async fn auth_or_logout(
		&self,
		state: &RouterState<Client>,
		jar: &CookieJar,
	) -> Result<UserInfo, Response> {
		match self.check_cookies(state, jar).await {
			Ok(None) => {}
			Ok(Some(u)) => return Ok(u),
			Err(e) => {
				error!(
					message = "Could not check auth cookies",
					cookies = ?jar,
					error = ?e
				);
				return Err((
					StatusCode::INTERNAL_SERVER_ERROR,
					Json("Could not check auth cookies"),
				)
					.into_response());
			}
		}

		// If cookie is invalid, clean up and delete client cookies
		let _ = self.terminate_session(jar).await;
		let cookie = Cookie::build((AUTH_COOKIE_NAME, ""))
			.path("/")
			.secure(true)
			.http_only(true)
			.same_site(SameSite::None)
			.expires(Expiration::from(OffsetDateTime::UNIX_EPOCH));

		return Err((
			StatusCode::UNAUTHORIZED,
			AppendHeaders([(SET_COOKIE, cookie.to_string())]),
			Json("Invalid auth cookie, logging out"),
		)
			.into_response());
	}
}
