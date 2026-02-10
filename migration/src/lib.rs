pub use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::TransactionTrait;

mod m20250415_031125_create_table_contacts;
mod m20250707_013515_create_table_users;
mod m20260201_000001_create_table_merchants;
mod m20260201_000002_create_table_admin_users;
mod m20260201_000003_create_table_audit_logs;
mod m20260201_000004_create_tenant_tables;
mod m20260201_000005_add_user_tenant_columns;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250415_031125_create_table_contacts::Migration),
            Box::new(m20250707_013515_create_table_users::Migration),
            Box::new(m20260201_000004_create_tenant_tables::Migration),
            Box::new(m20260201_000005_add_user_tenant_columns::Migration),
        ]
    }
}

pub struct PublicMigrator;

#[async_trait::async_trait]
impl MigratorTrait for PublicMigrator {
    fn migration_table_name() -> DynIden {
        "seaql_migrations_public".into_iden()
    }

    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260201_000001_create_table_merchants::Migration),
            Box::new(m20260201_000002_create_table_admin_users::Migration),
            Box::new(m20260201_000003_create_table_audit_logs::Migration),
        ]
    }
}

pub struct TenantMigrator;

#[async_trait::async_trait]
impl MigratorTrait for TenantMigrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250415_031125_create_table_contacts::Migration),
            Box::new(m20250707_013515_create_table_users::Migration),
            Box::new(m20260201_000004_create_tenant_tables::Migration),
            Box::new(m20260201_000005_add_user_tenant_columns::Migration),
        ]
    }
}

pub async fn run_tenant_migrations(
    connection: &sea_orm_migration::sea_orm::DatabaseConnection,
    schema: &str,
) -> Result<(), sea_orm_migration::sea_orm::DbErr> {
    if !schema
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Err(sea_orm_migration::sea_orm::DbErr::Custom(
            "Invalid schema name".to_string(),
        ));
    }

    let txn = connection.begin().await?;
    let stmt = sea_orm_migration::sea_orm::Statement::from_sql_and_values(
        sea_orm_migration::sea_orm::DatabaseBackend::Postgres,
        "SELECT set_config('search_path', $1, true)",
        vec![schema.to_string().into()],
    );
    sea_orm_migration::sea_orm::ConnectionTrait::execute(&txn, stmt).await?;
    TenantMigrator::up(&txn, None).await?;
    txn.commit().await
}
