use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::ConnectionTrait;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(r#"DROP INDEX IF EXISTS "idx_users_merchant_email_unique""#)
            .await?;
        manager
            .get_connection()
            .execute_unprepared(r#"DROP INDEX IF EXISTS "idx_users_merchant_phone_unique""#)
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                r#"CREATE UNIQUE INDEX IF NOT EXISTS "idx_users_email_unique" ON "users" ("email") WHERE "email" IS NOT NULL"#,
            )
            .await?;
        manager
            .get_connection()
            .execute_unprepared(
                r#"CREATE UNIQUE INDEX IF NOT EXISTS "idx_users_phone_unique" ON "users" ("phone_number") WHERE "phone_number" IS NOT NULL"#,
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(r#"DROP INDEX IF EXISTS "idx_users_email_unique""#)
            .await?;
        manager
            .get_connection()
            .execute_unprepared(r#"DROP INDEX IF EXISTS "idx_users_phone_unique""#)
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                r#"CREATE UNIQUE INDEX IF NOT EXISTS "idx_users_merchant_email_unique" ON "users" ("merchant_id", "email") WHERE "email" IS NOT NULL"#,
            )
            .await?;
        manager
            .get_connection()
            .execute_unprepared(
                r#"CREATE UNIQUE INDEX IF NOT EXISTS "idx_users_merchant_phone_unique" ON "users" ("merchant_id", "phone_number") WHERE "phone_number" IS NOT NULL"#,
            )
            .await?;

        Ok(())
    }
}
