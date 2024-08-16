use copper_migrate::Migration;

pub(super) struct InitMigration {}

#[async_trait::async_trait]
impl Migration for InitMigration {
	fn name(&self) -> &str {
		"init"
	}

	async fn up(&self, conn: &mut sqlx::SqliteConnection) -> Result<(), sqlx::Error> {
		sqlx::query(include_str!("./init.sql"))
			.execute(&mut *conn)
			.await?;

		sqlx::query("INSERT INTO meta (var, val) VALUES (?, ?);")
			.bind("copper_version")
			.bind(env!("CARGO_PKG_VERSION"))
			.execute(&mut *conn)
			.await?;

		return Ok(());
	}
}
