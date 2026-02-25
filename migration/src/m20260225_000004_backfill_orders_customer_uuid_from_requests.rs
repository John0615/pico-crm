use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::Statement;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let backend = manager.get_database_backend();
        let stmt = Statement::from_string(
            backend,
            r#"
            UPDATE orders o
            SET customer_uuid = sr.customer_uuid
            FROM service_requests sr
            WHERE o.customer_uuid IS NULL
              AND o.request_id IS NOT NULL
              AND o.request_id = sr.uuid;
            "#
            .to_string(),
        );
        manager.get_connection().execute(stmt).await?;
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
