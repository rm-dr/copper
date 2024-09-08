use copper_migrate::Migration;
use sqlx::Connection;

pub(super) struct MigrationStep {}

#[async_trait::async_trait]
impl Migration for MigrationStep {
	fn name(&self) -> &str {
		"m_0_init"
	}

	async fn up(&self, conn: &mut sqlx::SqliteConnection) -> Result<(), sqlx::Error> {
		let mut t = conn.begin().await?;

		sqlx::query(include_str!("./m_0_init.sql"))
			.execute(&mut *t)
			.await?;

		sqlx::query("INSERT INTO meta (var, val) VALUES (?, ?);")
			.bind("copper_version")
			.bind(env!("CARGO_PKG_VERSION"))
			.execute(&mut *t)
			.await?;

		t.commit().await?;

		return Ok(());
	}
}
