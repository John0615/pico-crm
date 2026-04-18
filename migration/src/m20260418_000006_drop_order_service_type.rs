use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::{ConnectionTrait, Statement};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let backend = manager.get_database_backend();
        manager
            .get_connection()
            .execute(Statement::from_string(
                backend,
                "DROP INDEX IF EXISTS idx_orders_service_type".to_string(),
            ))
            .await?;
        manager
            .get_connection()
            .execute(Statement::from_string(
                backend,
                "ALTER TABLE IF EXISTS orders DROP COLUMN IF EXISTS service_type".to_string(),
            ))
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Orders::Table)
                    .add_column_if_not_exists(ColumnDef::new(Orders::ServiceType).string().null())
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_orders_service_type")
                    .table(Orders::Table)
                    .col(Orders::ServiceType)
                    .if_not_exists()
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Orders {
    Table,
    ServiceType,
}
