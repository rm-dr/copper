-- Database metadata
CREATE TABLE meta (
	-- The name of the metadata variable
	var TEXT PRIMARY KEY NOT NULL UNIQUE,

	-- The value of the variable
	val TEXT NOT NULL
);

CREATE INDEX idx_meta_var on meta(var);



-- Datasets
CREATE TABLE dataset (
	id INTEGER PRIMARY KEY NOT NULL,

	-- This dataset's display name
	pretty_name TEXT NOT NULL UNIQUE
);


-- Item classes
CREATE TABLE class (
	id INTEGER PRIMARY KEY NOT NULL,

	-- The dataset this class belongs to
	dataset_id INTEGER NOT NULL,

	-- This class' display name
	pretty_name TEXT NOT NULL UNIQUE,

	FOREIGN KEY (dataset_id) REFERENCES dataset(id) ON DELETE CASCADE
);

CREATE INDEX idx_class_name on class(dataset_id, pretty_name);


-- Attribute metadata
CREATE TABLE attribute (
	id INTEGER PRIMARY KEY NOT NULL,

	-- The class this attribute belongs to
	class_id INTEGER NOT NULL,

	-- The order of this attribute in its class.
	-- Starts at 0, must be consecutive within each class.
	attr_order INTEGER NOT NULL,

	-- This attr's display name
	pretty_name TEXT NOT NULL,

	-- The type of data this attr holds
	data_type TEXT NOT NULL,

	--- Boolean (0 or 1). Does this attribute have a "unique" constraint?
	is_unique INTEGER NOT NULL,

	--- Boolean (0 or 1). Does this attribute have a "not_null" constraint?
	is_not_null INTEGER NOT NULL,

	FOREIGN KEY (class_id) REFERENCES class(id) ON DELETE CASCADE

	-- Attribute names must be unique within a class
	UNIQUE (pretty_name, class_id)
	UNIQUE (attr_order, class_id)
);

CREATE INDEX idx_attribute_class_name on attribute(class_id, pretty_name);


-- Items
CREATE TABLE item (
	id INTEGER PRIMARY KEY NOT NULL,

	-- The class this item belongs to
	class_id INTEGER NOT NULL,

	FOREIGN KEY (class_id) REFERENCES class(id) ON DELETE CASCADE
);

CREATE INDEX idx_item_class on item(class_id);


-- Attribute values
CREATE TABLE attribute_instance (
	-- The item to which this attribute is connected
	item_id INTEGER NOT NULL,

	-- The attribute this is an instance of
	attribute_id INTEGER NOT NULL,

	-- The value of this instance
	attribute_value TEXT NOT NULL,

	FOREIGN KEY (item_id) REFERENCES item(id) ON DELETE CASCADE
	FOREIGN KEY (attribute_id) REFERENCES attribute(id) ON DELETE CASCADE

	CONSTRAINT pk_attribute_instance PRIMARY KEY (item_id, attribute_id)
);

CREATE INDEX idx_attrinst_item on attribute_instance(item_id);
