use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Customers::Table)
                    .drop_column(Customers::LastContact)
                    .drop_column(Customers::ValueLevel)
                    .drop_column(Customers::Status)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Customers::Table)
                    .add_column_if_not_exists(
                        ColumnDef::new(Customers::LastContact)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(Customers::ValueLevel)
                            .integer()
                            .not_null()
                            .default(1),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(Customers::Status)
                            .integer()
                            .not_null()
                            .default(2),
                    )
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Customers {
    Table,
    LastContact,
    ValueLevel,
    Status,
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm_migration::sea_orm::DbBackend;

    #[test]
    fn drop_sql_removes_deprecated_customer_columns() {
        let sql = DbBackend::Postgres
            .build(
                &Table::alter()
                    .table(Customers::Table)
                    .drop_column(Customers::LastContact)
                    .drop_column(Customers::ValueLevel)
                    .drop_column(Customers::Status)
                    .to_owned(),
            )
            .to_string();

        assert!(sql.contains(r#"DROP COLUMN "last_contact""#));
        assert!(sql.contains(r#"DROP COLUMN "value_level""#));
        assert!(sql.contains(r#"DROP COLUMN "status""#));
    }
}
