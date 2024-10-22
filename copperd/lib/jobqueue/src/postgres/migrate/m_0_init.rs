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
			CREATE TABLE jobs (
				id TEXT PRIMARY KEY,

				created_at TIMESTAMPTZ NOT NULL,
				started_at TIMESTAMPTZ,
				finished_at TIMESTAMPTZ,

				owned_by BIGINT NOT NULL,
				state TEXT NOT NULL,

				pipeline JSONB NOT NULL,
				input JSONB NOT NULL
			);
			",
		)
		.execute(&mut *t)
		.await?;

		sqlx::query("CREATE INDEX idx_jobs_created_at on jobs(created_at);")
			.execute(&mut *t)
			.await?;

		sqlx::query("CREATE INDEX idx_jobs_owned_by on jobs(owned_by);")
			.execute(&mut *t)
			.await?;

		sqlx::query("CREATE INDEX idx_jobs_state on jobs(state);")
			.execute(&mut *t)
			.await?;

		t.commit().await?;

		return Ok(());
	}
}
