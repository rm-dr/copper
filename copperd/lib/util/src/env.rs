use serde::de::DeserializeOwned;
use smartstring::{LazyCompact, SmartString};
use std::{env::VarError, error::Error, fmt::Display, io::ErrorKind, path::PathBuf};

#[derive(Debug)]
pub enum EnvLoadError {
	IOError(std::io::Error),
	VarError(VarError),
	LineParse { on_line: String, at_char: usize },
	Other(dotenvy::Error),
	MissingValue(SmartString<LazyCompact>),
	OtherParseError(String),
}

impl Display for EnvLoadError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::IOError(_) => write!(f, "i/o error"),
			Self::VarError(_) => write!(f, "varerror"),
			Self::LineParse { on_line, at_char } => {
				write!(f, "line parse error: `{on_line}` at char {at_char}")
			}
			Self::Other(_) => write!(f, "other dotenvy error"),
			Self::MissingValue(v) => write!(f, "missing value {v}"),
			Self::OtherParseError(msg) => write!(f, "parse error: {msg}"),
		}
	}
}

impl Error for EnvLoadError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::IOError(e) => Some(e),
			Self::VarError(e) => Some(e),
			Self::Other(e) => Some(e),
			_ => None,
		}
	}
}

pub enum LoadedEnv<T> {
	/// We loaded config from `.env` and env vars
	FoundFile { config: T, path: PathBuf },

	/// We could not find `.env` and only loaded env vars
	OnlyVars(T),
}

impl<T> LoadedEnv<T> {
	pub fn get_config(&self) -> &T {
		match self {
			Self::FoundFile { config, .. } => config,
			Self::OnlyVars(config) => config,
		}
	}
}

/// Load the configuration type `T` from the current environment,
/// including the `.env` if it exists.
pub fn load_env<T: DeserializeOwned>() -> Result<LoadedEnv<T>, EnvLoadError> {
	let env_path = match dotenvy::dotenv() {
		Ok(path) => Some(path),

		Err(dotenvy::Error::Io(err)) => match err.kind() {
			ErrorKind::NotFound => None,
			_ => return Err(EnvLoadError::IOError(err)),
		},

		Err(dotenvy::Error::EnvVar(err)) => {
			return Err(EnvLoadError::VarError(err));
		}

		Err(dotenvy::Error::LineParse(on_line, at_char)) => {
			return Err(EnvLoadError::LineParse { on_line, at_char });
		}

		Err(err) => {
			return Err(EnvLoadError::Other(err));
		}
	};

	match envy::from_env::<T>() {
		Ok(config) => {
			if let Some(path) = env_path {
				return Ok(LoadedEnv::FoundFile { path, config });
			} else {
				return Ok(LoadedEnv::OnlyVars(config));
			}
		}

		Err(envy::Error::MissingValue(value)) => {
			return Err(EnvLoadError::MissingValue(value.into()))
		}

		Err(envy::Error::Custom(message)) => {
			return Err(EnvLoadError::OtherParseError(message));
		}
	};
}
