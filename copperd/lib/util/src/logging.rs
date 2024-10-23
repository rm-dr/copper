use std::{fmt::Display, str::FromStr};

use serde::Deserialize;
use tracing_subscriber::EnvFilter;

#[derive(Debug)]
pub enum LogLevel {
	Trace,
	Debug,
	Info,
	Warn,
	Error,
}

impl Default for LogLevel {
	fn default() -> Self {
		Self::Info
	}
}

impl Display for LogLevel {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Trace => write!(f, "trace"),
			Self::Debug => write!(f, "debug"),
			Self::Info => write!(f, "info"),
			Self::Warn => write!(f, "warn"),
			Self::Error => write!(f, "error"),
		}
	}
}

#[derive(Debug, Deserialize, Clone, Copy)]
pub enum LoggingPreset {
	Default,
	Verbose,
	Develop,
	Trace,
}

impl Default for LoggingPreset {
	fn default() -> Self {
		return Self::Default;
	}
}

impl LoggingPreset {
	pub fn get_config(&self) -> LoggingConfig {
		match self {
			Self::Default => LoggingConfig {
				other: LogLevel::Warn,
				http: LogLevel::Warn,
				s3: LogLevel::Warn,

				pipelined: LogLevel::Info,
				runner: LogLevel::Info,
				job: LogLevel::Info,
				nodes: LogLevel::Warn,

				edged: LogLevel::Info,
			},

			Self::Verbose => LoggingConfig {
				other: LogLevel::Warn,
				http: LogLevel::Warn,
				s3: LogLevel::Warn,

				pipelined: LogLevel::Debug,
				runner: LogLevel::Debug,
				job: LogLevel::Debug,
				nodes: LogLevel::Warn,

				edged: LogLevel::Debug,
			},

			Self::Develop => LoggingConfig {
				other: LogLevel::Debug,
				http: LogLevel::Warn,
				s3: LogLevel::Warn,

				pipelined: LogLevel::Trace,
				runner: LogLevel::Trace,
				job: LogLevel::Debug,
				nodes: LogLevel::Warn,

				edged: LogLevel::Trace,
			},

			Self::Trace => LoggingConfig {
				other: LogLevel::Trace,
				http: LogLevel::Warn,
				s3: LogLevel::Warn,

				pipelined: LogLevel::Trace,
				runner: LogLevel::Trace,
				job: LogLevel::Trace,
				nodes: LogLevel::Trace,

				edged: LogLevel::Trace,
			},
		}
	}
}

pub struct LoggingConfig {
	other: LogLevel,
	http: LogLevel,
	s3: LogLevel,

	pipelined: LogLevel,
	runner: LogLevel,
	job: LogLevel,
	nodes: LogLevel,

	edged: LogLevel,
}

impl From<LoggingConfig> for EnvFilter {
	fn from(conf: LoggingConfig) -> Self {
		EnvFilter::from_str(
			&[
				//
				// Non-configurable sources
				//
				format!("sqlx={}", LogLevel::Warn),
				format!("aws_sdk_s3={}", LogLevel::Warn),
				format!("aws_smithy_runtime={}", LogLevel::Warn),
				format!("aws_smithy_runtime_api={}", LogLevel::Warn),
				format!("aws_sigv4={}", LogLevel::Warn),
				format!("hyper={}", LogLevel::Warn),
				//
				// Configurable sources
				//
				format!("tower_http={}", conf.http),
				format!("s3={}", conf.s3),
				// // Pipelined
				format!("pipelined::pipeline::runner={}", conf.runner),
				format!("pipelined::pipeline::job={}", conf.job),
				format!("pipelined={}", conf.pipelined),
				// Node implementations
				format!("nodes_basic={}", conf.nodes),
				format!("nodes_audiofile={}", conf.nodes),
				// // Edged
				format!("edged={}", conf.edged),
				conf.other.to_string(),
			]
			.join(","),
		)
		.unwrap()
	}
}
