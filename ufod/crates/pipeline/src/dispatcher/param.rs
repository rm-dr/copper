use serde::{de::DeserializeOwned, Deserialize};
use smartstring::{LazyCompact, SmartString};
use std::collections::{BTreeMap, BTreeSet};

use crate::api::PipelineData;

/// The types of node parameters we accept
pub enum NodeParameterType<DataType: PipelineData> {
	/// Pipeline data
	Data {
		/// The type of data we contain
		data_type: <DataType as PipelineData>::DataStubType,
	},

	/// A type of pipeline data
	DataType,

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
		item_type: Box<NodeParameterType<DataType>>,
	},

	/// A map from `String` to parameter
	Map {
		/// The type of item this map holds
		value_type: Box<NodeParameterType<DataType>>,
	},
}

/// The types of node parameters we accept
#[derive(Debug, Clone, Deserialize)]
#[serde(bound = "DataType: DeserializeOwned")]
#[serde(untagged)]
pub enum NodeParameterValue<DataType: PipelineData> {
	/// Pipeline data
	///
	/// `DataType` MUST NOT be deserialized transparently,
	/// or it may be confused for other value types
	/// (Most notable, `String`).
	Data(DataType),

	/// A type of pipeline data
	///
	/// `DataStubType` MUST NOT be deserialized transparently,
	/// or it may be confused for other value types
	/// (Most notable, `String`).
	DataType(<DataType as PipelineData>::DataStubType),

	/// A plain string. This is used to carry the value of both
	/// `String` and `Enum` types. If an `Enum` parameter receives
	/// a string it doesn't recognize, an error should be thrown.
	String(SmartString<LazyCompact>),

	/// A list of parameters
	List(Vec<NodeParameterValue<DataType>>),

	/// A map from `String` to parameter
	Map(BTreeMap<SmartString<LazyCompact>, NodeParameterValue<DataType>>),
}

/// A description of one parameter a node accepts
pub struct NodeParameterSpec<DataType: PipelineData> {
	/// The name of this parameter
	pub param_name: SmartString<LazyCompact>,

	/// The type of this parameter
	pub param_type: NodeParameterType<DataType>,

	/// If true, this parameter is optional
	pub is_optional: bool,
}
