use copper_migrate::Migration;
use sqlx::Connection;

pub(super) struct MigrationStep {}

#[async_trait::async_trait]
impl Migration for MigrationStep {
	fn name(&self) -> &str {
		"m_1_userdetails"
	}

	async fn up(&self, conn: &mut sqlx::SqliteConnection) -> Result<(), sqlx::Error> {
		let mut t = conn.begin().await?;

		sqlx::query("ALTER TABLE users ADD user_email TEXT;")
			.execute(&mut *t)
			.await?;

		sqlx::query("ALTER TABLE users ADD user_color TEXT NOT NULL DEFAULT '#1C7ED6';")
			.execute(&mut *t)
			.await?;

		t.commit().await?;

		return Ok(());
	}
}
