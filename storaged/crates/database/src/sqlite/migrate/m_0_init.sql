-- Database metadata
CREATE TABLE meta (
	-- The name of the metadata variable
	var TEXT PRIMARY KEY NOT NULL UNIQUE,

	-- The value of the variable
	val TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_meta_var on meta(var);



-- Datasets
CREATE TABLE IF NOT EXISTS dataset (
	id INTEGER PRIMARY KEY NOT NULL,

	-- This dataset's display name
	pretty_name TEXT NOT NULL UNIQUE
);


-- Item classes
CREATE TABLE IF NOT EXISTS itemclass (
	id INTEGER PRIMARY KEY NOT NULL,

	-- The dataset this class belongs to
	dataset_id INTEGER NOT NULL,

	-- This itemclass' display name
	pretty_name TEXT NOT NULL UNIQUE,

	FOREIGN KEY (dataset_id) REFERENCES dataset(id)
);

CREATE INDEX IF NOT EXISTS idx_itemclass_name on itemclass(dataset_id, pretty_name);


-- Attribute metadata
CREATE TABLE IF NOT EXISTS attribute (
	id INTEGER PRIMARY KEY NOT NULL,

	-- The class this attribute belongs to
	itemclass_id INTEGER NOT NULL,

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

	FOREIGN KEY (itemclass_id) REFERENCES itemclass(id)

	-- Attribute names must be unique within a class
	UNIQUE (pretty_name, itemclass_id)
	UNIQUE (attr_order, itemclass_id)
);

CREATE INDEX IF NOT EXISTS idx_attribute_name on attribute(pretty_name);
CREATE INDEX IF NOT EXISTS idx_attribute_itemclass on attribute(itemclass_id);
