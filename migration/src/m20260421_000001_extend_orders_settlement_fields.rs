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
                        ColumnDef::new(Orders::PaidAmountCents).big_integer().null(),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(Orders::PaymentMethod).string_len(32).null(),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(Orders::PaidAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Orders::Table)
                    .drop_column(Orders::PaidAt)
                    .drop_column(Orders::PaymentMethod)
                    .drop_column(Orders::PaidAmountCents)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Orders {
    Table,
    PaidAmountCents,
    PaymentMethod,
    PaidAt,
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm_migration::sea_orm::DbBackend;

    #[test]
    fn alter_table_sql_includes_settlement_extension_fields() {
        let sql = DbBackend::Postgres
            .build(
                &Table::alter()
                    .table(Orders::Table)
                    .add_column_if_not_exists(
                        ColumnDef::new(Orders::PaidAmountCents).big_integer().null(),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(Orders::PaymentMethod).string_len(32).null(),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(Orders::PaidAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .to_owned(),
            )
            .to_string();

        assert!(sql.contains(r#"ADD COLUMN IF NOT EXISTS "paid_amount_cents" bigint"#));
        assert!(sql.contains(r#"ADD COLUMN IF NOT EXISTS "payment_method" varchar(32)"#));
        assert!(sql.contains(r#"ADD COLUMN IF NOT EXISTS "paid_at" timestamp with time zone"#));
    }
}
