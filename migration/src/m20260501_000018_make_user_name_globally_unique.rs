use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::ConnectionTrait;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(r#"DROP INDEX IF EXISTS "idx_users_merchant_username_unique""#)
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                r#"CREATE UNIQUE INDEX IF NOT EXISTS "idx_users_username_unique" ON "users" ("user_name")"#,
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(r#"DROP INDEX IF EXISTS "idx_users_username_unique""#)
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                r#"CREATE UNIQUE INDEX IF NOT EXISTS "idx_users_merchant_username_unique" ON "users" ("merchant_id", "user_name")"#,
            )
            .await?;

        Ok(())
    }
}
