use sea_orm_migration::prelude::*;

pub struct Migration;

// sea-orm-cli generate entity -u "sqlite:./test.sqlite?mode=rwc" -o src/entries

impl MigrationName for Migration {
	fn name(&self) -> &str {
		"m_20220602_000001_create_model"
	}
}

#[derive(Iden)]
pub enum Class {
	Table,
	Id,
	Name,
}

#[derive(Iden)]
pub enum Attr {
	Table,
	Id,
	Name,
	Class,
	Datatype,
}

#[derive(Iden)]
pub enum AttrDatatype {
	String,
}

#[derive(Iden)]
pub enum Item {
	Table,
	Id,
	Class,
}

#[derive(Iden)]
pub enum ValueStr {
	Table,
	Id,
	Attr,
	Item,
	Value,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	/*
	// Override the name of migration table
	fn migration_table_name() -> sea_orm::DynIden {
		Alias::new("override_migration_table_name").into_iden()
	}
	*/

	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.create_table(
				Table::create()
					.table(Class::Table)
					.col(
						ColumnDef::new(Class::Id)
							.integer()
							.not_null()
							.auto_increment()
							.primary_key(),
					)
					.col(ColumnDef::new(Class::Name).string().not_null())
					.to_owned(),
			)
			.await?;

		manager
			.create_table(
				Table::create()
					.table(Attr::Table)
					.col(
						ColumnDef::new(Attr::Id)
							.integer()
							.not_null()
							.auto_increment()
							.primary_key(),
					)
					.col(ColumnDef::new(Attr::Name).string().not_null())
					.col(ColumnDef::new(Attr::Class).integer().not_null())
					.foreign_key(
						ForeignKey::create()
							.name("fk-attr-class")
							.from(Attr::Table, Attr::Class)
							.to(Class::Table, Class::Id),
					)
					.col(
						ColumnDef::new(Attr::Datatype)
							.enumeration(Attr::Datatype, [AttrDatatype::String])
							.not_null(),
					)
					.to_owned(),
			)
			.await?;

		manager
			.create_table(
				Table::create()
					.table(Item::Table)
					.col(
						ColumnDef::new(Item::Id)
							.integer()
							.not_null()
							.auto_increment()
							.primary_key(),
					)
					.col(ColumnDef::new(Item::Class).integer().not_null())
					.foreign_key(
						ForeignKey::create()
							.name("fk-item-class")
							.from(Item::Table, Item::Class)
							.to(Class::Table, Class::Id),
					)
					.to_owned(),
			)
			.await?;

		manager
			.create_table(
				Table::create()
					.table(ValueStr::Table)
					.col(
						ColumnDef::new(ValueStr::Id)
							.integer()
							.not_null()
							.auto_increment()
							.primary_key(),
					)
					.col(ColumnDef::new(ValueStr::Value).string().not_null())
					.col(ColumnDef::new(ValueStr::Attr).integer().not_null())
					.foreign_key(
						ForeignKey::create()
							.name("fk-strvalue-attr")
							.from(ValueStr::Table, ValueStr::Attr)
							.to(Attr::Table, Attr::Id),
					)
					.col(ColumnDef::new(ValueStr::Item).integer().not_null())
					.foreign_key(
						ForeignKey::create()
							.name("fk-strvalue-item")
							.from(ValueStr::Table, ValueStr::Item)
							.to(Item::Table, Item::Id),
					)
					.to_owned(),
			)
			.await?;

		Ok(())
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		// Order here matters!
		// Watch out for foreign keys.

		manager
			.drop_table(Table::drop().table(ValueStr::Table).to_owned())
			.await?;
		manager
			.drop_table(Table::drop().table(Attr::Table).to_owned())
			.await?;
		manager
			.drop_table(Table::drop().table(Item::Table).to_owned())
			.await?;
		manager
			.drop_table(Table::drop().table(Class::Table).to_owned())
			.await?;
		Ok(())
	}
}
