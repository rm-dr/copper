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

		sqlx::query(
			"
			CREATE TABLE meta (
				var TEXT PRIMARY KEY NOT NULL UNIQUE,
				val TEXT NOT NULL
			);
			",
		)
		.execute(&mut *t)
		.await?;

		sqlx::query("CREATE INDEX idx_meta_var on meta(var);")
			.execute(&mut *t)
			.await?;

		sqlx::query("INSERT INTO meta (var, val) VALUES ($1, $2);")
			.bind("copper_version")
			.bind(env!("CARGO_PKG_VERSION"))
			.execute(&mut *t)
			.await?;

		//
		// MARK: users
		//

		sqlx::query(
			"
			CREATE TABLE users (
				id BIGSERIAL PRIMARY KEY,
				user_email TEXT NOT NULL UNIQUE,
				user_name TEXT NOT NULL,
				user_pass JSON NOT NULL
			);
			",
		)
		.execute(&mut *t)
		.await?;

		sqlx::query("CREATE INDEX user_email on users(user_email);")
			.execute(&mut *t)
			.await?;

		//
		// MARK: pipeline
		//

		sqlx::query(
			"
			CREATE TABLE pipelines (
				id BIGSERIAL PRIMARY KEY,
				owned_by BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
				name TEXT NOT NULL,
				data JSON NOT NULL
			);
			",
		)
		.execute(&mut *t)
		.await?;

		sqlx::query("CREATE UNIQUE INDEX pipeline_user_name on pipelines(owned_by, name);")
			.execute(&mut *t)
			.await?;

		t.commit().await?;

		return Ok(());
	}
}
