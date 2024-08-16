use copper_migrate::Migration;

mod m_0_init;

pub const MIGRATE_STEPS: &'static [&'static dyn Migration] = &[&m_0_init::MigrationStep {}];
