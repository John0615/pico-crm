use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::{ConnectionTrait, Statement};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column_if_not_exists(
                        ColumnDef::new(Users::TrainingRecords)
                            .json_binary()
                            .not_null()
                            .default(Expr::cust("'[]'::jsonb")),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(Users::Certificates)
                            .json_binary()
                            .not_null()
                            .default(Expr::cust("'[]'::jsonb")),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(Users::HealthStatus)
                            .string()
                            .not_null()
                            .default("healthy"),
                    )
                    .to_owned(),
            )
            .await?;

        let backend = manager.get_database_backend();
        manager
            .get_connection()
            .execute(Statement::from_string(
                backend,
                "UPDATE users SET training_records = '[]'::jsonb WHERE training_records IS NULL"
                    .to_string(),
            ))
            .await?;
        manager
            .get_connection()
            .execute(Statement::from_string(
                backend,
                "UPDATE users SET certificates = '[]'::jsonb WHERE certificates IS NULL"
                    .to_string(),
            ))
            .await?;
        manager
            .get_connection()
            .execute(Statement::from_string(
                backend,
                "UPDATE users SET health_status = 'healthy' WHERE health_status IS NULL OR health_status = ''"
                    .to_string(),
            ))
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .drop_column(Users::HealthStatus)
                    .drop_column(Users::Certificates)
                    .drop_column(Users::TrainingRecords)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    TrainingRecords,
    Certificates,
    HealthStatus,
}
