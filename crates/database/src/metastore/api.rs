use smartstring::{LazyCompact, SmartString};

use super::{
	data::{MetastoreData, MetastoreDataStub},
	errors::MetastoreError,
	handles::{AttrHandle, ClassHandle, ItemHandle},
};

pub struct AttributeOptions {
	pub(crate) unique: bool,
	pub(crate) not_null: bool,
}

#[allow(clippy::derivable_impls)]
impl Default for AttributeOptions {
	fn default() -> Self {
		Self {
			unique: false,
			not_null: false,
		}
	}
}

impl AttributeOptions {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn unique(mut self, is_unique: bool) -> Self {
		self.unique = is_unique;
		self
	}

	pub fn not_null(mut self, not_null: bool) -> Self {
		self.not_null = not_null;
		self
	}
}

pub trait Metastore
where
	Self: Send,
{
	fn add_class(&mut self, name: &str) -> Result<ClassHandle, MetastoreError>;
	fn add_item(
		&mut self,
		class: ClassHandle,
		attrs: Vec<(AttrHandle, MetastoreData)>,
	) -> Result<ItemHandle, MetastoreError>;
	fn add_attr(
		&mut self,
		class: ClassHandle,
		name: &str,
		data_type: MetastoreDataStub,
		options: AttributeOptions,
	) -> Result<AttrHandle, MetastoreError>;

	fn del_class(&mut self, class: ClassHandle) -> Result<(), MetastoreError>;
	fn del_item(&mut self, item: ItemHandle) -> Result<(), MetastoreError>;
	fn del_attr(&mut self, attr: AttrHandle) -> Result<(), MetastoreError>;

	//fn iter_items(&self) -> Result<impl Iterator<Item = ItemHandle>, ()>;
	//fn iter_classes(&self) -> Result<impl Iterator<Item = ClassHandle>, ()>;
	//fn iter_attrs(&self) -> Result<impl Iterator<Item = AttrHandle>, ()>;

	fn get_class(&mut self, class_name: &str) -> Result<Option<ClassHandle>, MetastoreError>;
	fn get_attr(
		&mut self,
		class: ClassHandle,
		attr_name: &str,
	) -> Result<Option<AttrHandle>, MetastoreError>;

	// TODO: take &[(_, _)] instead of single data
	fn item_set_attr(&mut self, attr: AttrHandle, data: MetastoreData) -> Result<(), MetastoreError>;
	fn item_get_attr(
		&mut self,
		item: ItemHandle,
		attr: AttrHandle,
	) -> Result<MetastoreData, MetastoreError>;
	fn item_get_class(&mut self, item: ItemHandle) -> Result<ClassHandle, MetastoreError>;

	fn class_set_name(&mut self, class: ClassHandle, name: &str) -> Result<(), MetastoreError>;
	fn class_get_name(&mut self, class: ClassHandle) -> Result<&str, MetastoreError>;

	/// Get all attributes in the given class.
	/// Returns (attr handle, attr name, attr type)
	///
	/// Attribute order MUST be consistent!
	fn class_get_attrs(
		&mut self,
		class: ClassHandle,
	) -> Result<Vec<(AttrHandle, SmartString<LazyCompact>, MetastoreDataStub)>, MetastoreError>;
	fn class_num_attrs(&mut self, class: ClassHandle) -> Result<usize, MetastoreError>;

	fn attr_set_name(&mut self, attr: AttrHandle, name: &str) -> Result<(), MetastoreError>;
	fn attr_get_name(&mut self, attr: AttrHandle) -> Result<&str, MetastoreError>;
	fn attr_get_type(&mut self, attr: AttrHandle) -> Result<MetastoreDataStub, MetastoreError>;
	fn attr_get_class(&mut self, attr: AttrHandle) -> ClassHandle;

	fn find_item_with_attr(
		&mut self,
		attr: AttrHandle,
		attr_value: MetastoreData,
	) -> Result<Option<ItemHandle>, MetastoreError>;
}
