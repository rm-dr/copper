CREATE TABLE IF NOT EXISTS meta_classes (
	id INTEGER PRIMARY KEY NOT NULL,
	table_name TEXT NOT NULL UNIQUE,
	pretty_name TEXT NOT NULL
);


CREATE TABLE IF NOT EXISTS meta_attributes (
	id INTEGER PRIMARY KEY NOT NULL,
	class_id INTEGER,
	column_name TEXT NOT NULL,
	pretty_name TEXT NOT NULL,
	data_type TEXT NOT NULL,
	is_unique INTEGER NOT NULL,
	is_not_null INTEGER NOT NULL,
	FOREIGN KEY (class_id) REFERENCES meta_classes(id)
	UNIQUE (column_name, class_id)
	UNIQUE (pretty_name, class_id)
);
