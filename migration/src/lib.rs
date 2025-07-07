pub use sea_orm_migration::prelude::*;

mod m20250415_031125_create_table_contacts;
mod m20250707_013515_create_table_users;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250415_031125_create_table_contacts::Migration),
            Box::new(m20250707_013515_create_table_users::Migration),
        ]
    }
}
