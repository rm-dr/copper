use serde::{de::Visitor, Deserialize};
use smartstring::{LazyCompact, SmartString};
use std::{fmt::Display, str::FromStr};

/// An output port in the pipeline.
/// (i.e, a port that produces data.)
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum PipelineOutput {
	/// A pipeline input
	Pinput { port: SmartString<LazyCompact> },

	/// An output port of a node
	Node {
		node: SmartString<LazyCompact>,
		port: SmartString<LazyCompact>,
	},

	/// Inline static text
	InlineText { text: String },
}

impl PipelineOutput {
	pub fn node_str(&self) -> Option<&str> {
		match self {
			Self::Pinput { .. } => Some("in"),
			Self::Node { node, .. } => Some(node),
			Self::InlineText { .. } => None,
		}
	}

	pub fn port_str(&self) -> Option<&str> {
		match self {
			Self::Pinput { port } | Self::Node { port, .. } => Some(port),
			Self::InlineText { .. } => None,
		}
	}
}

impl Display for PipelineOutput {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Pinput { port } => write!(f, "in.{}", port),
			Self::Node { node, port } => write!(f, "{}.{}", node, port),
			Self::InlineText { text } => write!(f, "InlineText({text})"),
		}
	}
}

// TODO: better error
impl FromStr for PipelineOutput {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let mut i = s.split('.');
		let a = i.next();
		let b = i.next();

		if a.is_none() || b.is_none() || i.next().is_some() {
			return Err("bad link format".into());
		}
		let a = a.unwrap();
		let b = b.unwrap();

		Ok(match a {
			"in" => Self::Pinput { port: b.into() },
			//"out" => Self::Poutput { port: b.into() },
			_ => Self::Node {
				node: a.into(),
				port: b.into(),
			},
		})
	}
}
struct PipelineOutputVisitor;
impl<'de> Visitor<'de> for PipelineOutputVisitor {
	type Value = PipelineOutput;

	fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
		formatter.write_str("an integer between -2^31 and 2^31")
	}

	fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
	where
		E: serde::de::Error,
	{
		let s = PipelineOutput::from_str(v);
		s.map_err(|x| serde::de::Error::custom(x))
	}

	fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
	where
		A: serde::de::MapAccess<'de>,
	{
		let a: Option<(String, String)> = map.next_entry()?;
		if a.is_none() || map.next_key::<String>()?.is_some() {
			return Err(serde::de::Error::custom("bad inline"));
		}
		let a = a.unwrap();

		match &a.0[..] {
			"text" => Ok(PipelineOutput::InlineText { text: a.1 }),
			_ => return Err(serde::de::Error::custom("bad inline")),
		}
	}
}

impl<'de> Deserialize<'de> for PipelineOutput {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		deserializer.deserialize_any(PipelineOutputVisitor)
	}
}

/// An input port in the pipeline.
/// (i.e, a port that consumes data.)
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum PipelineInput {
	/// A pipeline output
	Poutput { port: SmartString<LazyCompact> },

	/// An input port of a node
	Node {
		node: SmartString<LazyCompact>,
		port: SmartString<LazyCompact>,
	},
}

impl PipelineInput {
	pub fn node_str(&self) -> &str {
		match self {
			Self::Poutput { .. } => "out",
			Self::Node { node, .. } => node,
		}
	}

	pub fn port_str(&self) -> &str {
		match self {
			Self::Poutput { port } | Self::Node { port, .. } => port,
		}
	}
}

impl Display for PipelineInput {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Poutput { port } => write!(f, "in.{}", port),
			Self::Node { node, port } => write!(f, "{}.{}", node, port),
		}
	}
}

// TODO: better error
impl FromStr for PipelineInput {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let mut i = s.split('.');
		let a = i.next();
		let b = i.next();

		if a.is_none() || b.is_none() || i.next().is_some() {
			return Err("bad link format".into());
		}
		let a = a.unwrap();
		let b = b.unwrap();

		Ok(match a {
			//"in" => Self::Pinput { port: b.into() },
			"out" => Self::Poutput { port: b.into() },
			_ => Self::Node {
				node: a.into(),
				port: b.into(),
			},
		})
	}
}

impl<'de> Deserialize<'de> for PipelineInput {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		let addr_str = SmartString::<LazyCompact>::deserialize(deserializer)?;
		let s = Self::from_str(&addr_str);
		s.map_err(serde::de::Error::custom)
	}
}
