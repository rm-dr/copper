use std::io::ErrorKind;

use serde::de::DeserializeOwned;

/// Load the configuration type `T` from the current environment,
/// including the `.env` if it exists.
///
/// This method calls `std::process::exit` if loading fails.
pub fn load_env<T: DeserializeOwned>() -> T {
	match dotenvy::dotenv() {
		Ok(path) => {
			println!("Loaded env from {path:?}")
		}

		Err(dotenvy::Error::Io(err)) => match err.kind() {
			ErrorKind::NotFound => {
				println!("Could not find .env, not loading");
			}

			_ => {
				println!("ERROR: I/O error when loading .env: {err:?}");
				std::process::exit(1);
			}
		},

		Err(dotenvy::Error::EnvVar(err)) => {
			println!("ERROR: VarError when loading .env: {err:?}");
			std::process::exit(1);
		}

		Err(dotenvy::Error::LineParse(x, y)) => {
			println!("ERROR: Line parse error when loading .env");
			println!("On line: `{x}`");
			println!("At char:  {}^", " ".repeat(y));
			std::process::exit(1);
		}

		Err(err) => {
			println!("ERROR: Error while loading .env: {err:?}");
			std::process::exit(1);
		}
	};

	match envy::from_env::<T>() {
		Ok(config) => return config,

		Err(envy::Error::MissingValue(value)) => {
			println!("ERROR: Required env var {value} is missing");
			std::process::exit(1);
		}

		Err(envy::Error::Custom(message)) => {
			println!("ERROR: Could not parse config from env: {message}");
			std::process::exit(1);
		}
	};
}
