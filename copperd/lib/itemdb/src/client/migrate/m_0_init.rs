use copper_migrate::Migration;
use sqlx::Connection;

pub(super) struct MigrationStep {}

#[async_trait::async_trait]
impl Migration for MigrationStep {
	fn name(&self) -> &str {
		"m_0_init"
	}

	async fn up(&self, conn: &mut sqlx::PgConnection) -> Result<(), sqlx::Error> {
		let mut t = conn.begin().await?;

		//
		// MARK: Datasets
		//
		sqlx::query(
			"CREATE TABLE dataset (
				id BIGSERIAL PRIMARY KEY,

				-- This dataset's display name
				pretty_name TEXT NOT NULL,

				-- The id of the user that owns this dataset
				owner BIGINT NOT NULL
			);",
		)
		.execute(&mut *t)
		.await?;

		sqlx::query("CREATE UNIQUE INDEX idx_dataset_name_owner on dataset(pretty_name, owner);")
			.execute(&mut *t)
			.await?;

		//
		// MARK: Classes
		//

		sqlx::query(
			"CREATE TABLE class (
				id BIGSERIAL PRIMARY KEY,

				-- The dataset this class belongs to
				dataset_id BIGINT NOT NULL REFERENCES dataset(id) ON DELETE CASCADE,

				-- This class' display name
				pretty_name TEXT NOT NULL
			);",
		)
		.execute(&mut *t)
		.await?;

		sqlx::query("CREATE UNIQUE INDEX idx_class_name on class(dataset_id, pretty_name);")
			.execute(&mut *t)
			.await?;

		//
		// MARK: Attributes
		//

		sqlx::query(
			"CREATE TABLE attribute (
				id BIGSERIAL PRIMARY KEY,

				-- The class this attribute belongs to
				class_id BIGINT NOT NULL REFERENCES class(id) ON DELETE CASCADE,

				-- The order of this attribute in its class.
				-- Starts at 0, must be consecutive within each class.
				attr_order BIGINT NOT NULL,

				-- This attr's display name
				pretty_name TEXT NOT NULL,

				-- The type of data this attr holds
				data_type TEXT NOT NULL,

				--- Does this attribute have a \"unique\" constraint?
				is_unique BOOLEAN NOT NULL,

				--- Does this attribute have a \"not_null\" constraint?
				is_not_null BOOLEAN NOT NULL
			);",
		)
		.execute(&mut *t)
		.await?;

		sqlx::query(
			"CREATE UNIQUE INDEX idx_attribute_class_name on attribute(class_id, pretty_name);",
		)
		.execute(&mut *t)
		.await?;

		sqlx::query(
			"CREATE UNIQUE INDEX idx_attribute_order_class on attribute(attr_order, class_id);",
		)
		.execute(&mut *t)
		.await?;

		//
		// MARK: Items
		//

		sqlx::query(
			"CREATE TABLE item (
				id BIGSERIAL PRIMARY KEY,

				-- The class this item belongs to
				class_id BIGINT NOT NULL REFERENCES class(id) ON DELETE CASCADE
			);",
		)
		.execute(&mut *t)
		.await?;

		sqlx::query("CREATE INDEX idx_item_class on item(class_id);")
			.execute(&mut *t)
			.await?;

		//
		// MARK: Attribute instances
		//

		sqlx::query(
			"CREATE TABLE attribute_instance (
				-- The item to which this attribute is connected
				item_id BIGINT NOT NULL REFERENCES item(id) ON DELETE CASCADE,

				-- The attribute this is an instance of
				attribute_id BIGINT NOT NULL REFERENCES attribute(id) ON DELETE CASCADE,

				-- The value of this instance
				attribute_value TEXT NOT NULL,

				PRIMARY KEY (item_id, attribute_id)
			);",
		)
		.execute(&mut *t)
		.await?;

		sqlx::query("CREATE INDEX idx_attrinst_item on attribute_instance(item_id);")
			.execute(&mut *t)
			.await?;

		sqlx::query("CREATE INDEX idx_attrinst_attr on attribute_instance(attribute_id);")
			.execute(&mut *t)
			.await?;

		sqlx::query("CREATE INDEX idx_attrinst_value on attribute_instance(attribute_value);")
			.execute(&mut *t)
			.await?;

		//
		// MARK: finish
		//

		t.commit().await?;

		return Ok(());
	}
}
