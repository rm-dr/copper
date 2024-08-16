use copper_migrate::Migration;

mod m_0_init;
mod m_1_userdetails;

pub const MIGRATE_STEPS: &'static [&'static dyn Migration] = &[
	&m_0_init::MigrationStep {},
	&m_1_userdetails::MigrationStep {},
];
