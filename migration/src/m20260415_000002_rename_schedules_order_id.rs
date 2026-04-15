use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::{ConnectionTrait, Statement};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let stmt = Statement::from_string(
            manager.get_database_backend(),
            r#"
            DO $$
            BEGIN
                IF EXISTS (
                    SELECT 1
                    FROM information_schema.columns
                    WHERE table_name = 'schedules' AND column_name = 'order_id'
                ) AND NOT EXISTS (
                    SELECT 1
                    FROM information_schema.columns
                    WHERE table_name = 'schedules' AND column_name = 'order_uuid'
                ) THEN
                    ALTER TABLE schedules RENAME COLUMN order_id TO order_uuid;
                END IF;
            END $$;
            "#
            .to_string(),
        );
        manager.get_connection().execute(stmt).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let stmt = Statement::from_string(
            manager.get_database_backend(),
            r#"
            DO $$
            BEGIN
                IF EXISTS (
                    SELECT 1
                    FROM information_schema.columns
                    WHERE table_name = 'schedules' AND column_name = 'order_uuid'
                ) AND NOT EXISTS (
                    SELECT 1
                    FROM information_schema.columns
                    WHERE table_name = 'schedules' AND column_name = 'order_id'
                ) THEN
                    ALTER TABLE schedules RENAME COLUMN order_uuid TO order_id;
                END IF;
            END $$;
            "#
            .to_string(),
        );
        manager.get_connection().execute(stmt).await?;
        Ok(())
    }
}
