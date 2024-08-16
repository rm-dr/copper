use copper_migrate::Migration;

mod init;

pub const MIGRATE_STEPS: &'static [&'static dyn Migration] = &[&init::InitMigration {}];
