-- Database metadata
CREATE TABLE IF NOT EXISTS meta_meta (
	var TEXT NOT NULL UNIQUE,
	val TEXT NOT NULL
);

-- Class metadata
CREATE TABLE IF NOT EXISTS meta_classes (
	id INTEGER PRIMARY KEY NOT NULL,

	-- The name of this class' table in the db
	-- (sanitized)
	table_name TEXT NOT NULL UNIQUE,

	-- This class' display name
	pretty_name TEXT NOT NULL
);


-- Attribute metadata
CREATE TABLE IF NOT EXISTS meta_attributes (
	id INTEGER PRIMARY KEY NOT NULL,

	-- The class this attribute belongs to
	class_id INTEGER,

	-- The name of this attr's column in its class' table
	column_name TEXT NOT NULL,

	-- This attr's display name
	pretty_name TEXT NOT NULL,

	-- The type of data this attr holds
	-- (Internal UFO datatype. This is more specific than the types SQL provides.)
	data_type TEXT NOT NULL,

	--- Boolean (0 or 1). Does this attribute have a "unique" constrait?
	is_unique INTEGER NOT NULL,

	--- Boolean (0 or 1). Does this attribute have a "not_null" constrait?
	is_not_null INTEGER NOT NULL,

	FOREIGN KEY (class_id) REFERENCES meta_classes(id)

	UNIQUE (column_name, class_id) -- Attribute names must be unique within a class
	-- `column_name` is a function of `pretty_name`, so this is somewhat redundant...
	-- but we keep the constraint anyway, just in case.
	UNIQUE (pretty_name, class_id)
);
