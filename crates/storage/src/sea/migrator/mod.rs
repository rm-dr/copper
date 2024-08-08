use sea_orm_migration::prelude::*;

mod m20240501_000001_create_model;

pub use m20240501_000001_create_model::AttrDatatype;
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
	fn migrations() -> Vec<Box<dyn MigrationTrait>> {
		vec![Box::new(m20240501_000001_create_model::Migration)]
	}
}