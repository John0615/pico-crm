use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Orders::Table)
                    .add_column_if_not_exists(
                        ColumnDef::new(Orders::CancellationReason).text().null(),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(Orders::CompletedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_orders_completed_at")
                    .table(Orders::Table)
                    .col(Orders::CompletedAt)
                    .if_not_exists()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_orders_completed_at")
                    .table(Orders::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Orders::Table)
                    .drop_column(Orders::CompletedAt)
                    .drop_column(Orders::CancellationReason)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Orders {
    Table,
    CancellationReason,
    CompletedAt,
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm_migration::sea_orm::DbBackend;

    #[test]
    fn alter_table_sql_includes_t05_order_fields() {
        let sql = DbBackend::Postgres
            .build(
                &Table::alter()
                    .table(Orders::Table)
                    .add_column_if_not_exists(
                        ColumnDef::new(Orders::CancellationReason).text().null(),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(Orders::CompletedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .to_owned(),
            )
            .to_string();

        assert!(sql.contains(r#"ADD COLUMN IF NOT EXISTS "cancellation_reason" text"#));
        assert!(sql.contains(r#"ADD COLUMN IF NOT EXISTS "completed_at" timestamp with time zone"#));
    }
}
