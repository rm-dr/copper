-- Database metadata
CREATE TABLE meta_meta (
	-- The name of the metadata variable
	var TEXT PRIMARY KEY NOT NULL UNIQUE,

	-- The value of the variable
	val TEXT NOT NULL
);




-- Blob metadata
CREATE TABLE meta_blobs (
	id INTEGER PRIMARY KEY NOT NULL,

	-- This blob's mime type
	data_type TEXT NOT NULL,

	-- A relative path to this blob's file
	file_path TEXT NOT NULL UNIQUE
);




-- Pipelines
CREATE TABLE IF NOT EXISTS meta_pipelines (
	id INTEGER PRIMARY KEY NOT NULL,

	-- This pipeline's name
	pipeline_name TEXT UNIQUE NOT NULL,

	-- This pipeline, serialized
	pipeline_data TEXT UNIQUE NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_meta_pipeline_name on meta_pipelines(pipeline_name);




-- Class metadata
CREATE TABLE IF NOT EXISTS meta_classes (
	id INTEGER PRIMARY KEY NOT NULL,

	-- This class' display name
	pretty_name TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_meta_class_name on meta_classes(pretty_name);




-- Attribute metadata
CREATE TABLE IF NOT EXISTS meta_attributes (
	id INTEGER PRIMARY KEY NOT NULL,

	-- The class this attribute belongs to
	class_id INTEGER NOT NULL,

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

	-- Attribute names must be unique within a class
	UNIQUE (pretty_name, class_id)
);

CREATE INDEX IF NOT EXISTS idx_meta_attr_name on meta_attributes(pretty_name);
CREATE INDEX IF NOT EXISTS idx_meta_attr_class on meta_attributes(class_id);




-- Enum metadata
CREATE TABLE IF NOT EXISTS meta_enums (
	id INTEGER PRIMARY KEY NOT NULL,

	-- The item class this enum belongs to
	class_id INTEGER NOT NULL,

	-- This enum's display name
	pretty_name TEXT NOT NULL,

	-- Boolean (0 or 1). Can we select multiple variants of this enum at once?
	is_multi INTEGER NOT NULL,

	FOREIGN KEY (class_id) REFERENCES meta_classes(id)
	UNIQUE (pretty_name, class_id)
);

CREATE INDEX IF NOT EXISTS idx_meta_enum_name on meta_enums(pretty_name);
CREATE INDEX IF NOT EXISTS idx_meta_enum_class on meta_enums(class_id);




-- Enum variants
CREATE TABLE IF NOT EXISTS meta_enum_variants (
	id INTEGER PRIMARY KEY NOT NULL,

	-- The enum this variant belongs to
	enum_id INTEGER NOT NULL,

	-- This variant's display name
	pretty_name TEXT NOT NULL,

	FOREIGN KEY (enum_id) REFERENCES meta_enums(id)
	UNIQUE (pretty_name, enum_id)
);

CREATE INDEX IF NOT EXISTS idx_meta_enum_variant_name on meta_enum_variants(pretty_name);
CREATE INDEX IF NOT EXISTS idx_meta_enum_variant_enum on meta_enum_variants(enum_id);

