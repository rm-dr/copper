use serde::Deserialize;
use smartstring::{LazyCompact, SmartString};
use std::collections::{BTreeMap, BTreeSet};
use utoipa::ToSchema;

/// The types of node parameters we accept
pub enum NodeParameterType {
	/// A type of pipeline data
	DataType,

	/// A yes or a no
	Boolean,

	/// A plain string
	String,

	/// One of many predefined strings
	Enum {
		/// The values this enum can take
		variants: BTreeSet<SmartString<LazyCompact>>,
	},

	/// A list of parameters
	List {
		/// The type of item this list holds
		item_type: Box<NodeParameterType>,
	},

	/// A map from `String` to parameter
	Map {
		/// The type of item this map holds
		value_type: Box<NodeParameterType>,
	},
}

/// The types of node parameters we accept
#[derive(Debug, Clone, Deserialize, ToSchema)]
#[serde(tag = "parameter_type", content = "value")]
pub enum NodeParameterValue {
	/// A yes or a no
	Boolean(bool),

	/// An integer
	Integer(i64),

	/// A plain string. This is used to carry the value of both
	/// `String` and `Enum` types. If an `Enum` parameter receives
	/// a string it doesn't recognize, an error should be thrown.
	#[schema(value_type = String)]
	String(SmartString<LazyCompact>),

	/// A list of parameters
	List(Vec<NodeParameterValue>),

	/// A map from `String` to parameter
	#[schema(value_type = BTreeMap<String, NodeParameterValue>)]
	Map(BTreeMap<SmartString<LazyCompact>, NodeParameterValue>),
}

/// A description of one parameter a node accepts
pub struct NodeParameterSpec {
	/// The type of this parameter
	pub param_type: NodeParameterType,

	/// If true, this parameter is optional
	pub is_optional: bool,
}
