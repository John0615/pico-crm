use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(OrderChangeLogs::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(OrderChangeLogs::Uuid)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(OrderChangeLogs::OrderUuid).uuid().not_null())
                    .col(ColumnDef::new(OrderChangeLogs::Action).string().not_null())
                    .col(ColumnDef::new(OrderChangeLogs::OperatorUuid).uuid().null())
                    .col(ColumnDef::new(OrderChangeLogs::BeforeData).json_binary().null())
                    .col(ColumnDef::new(OrderChangeLogs::AfterData).json_binary().null())
                    .col(
                        ColumnDef::new(OrderChangeLogs::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_order_change_logs_order")
                            .from(OrderChangeLogs::Table, OrderChangeLogs::OrderUuid)
                            .to(Orders::Table, Orders::Uuid)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_order_change_logs_order_uuid")
                    .table(OrderChangeLogs::Table)
                    .col(OrderChangeLogs::OrderUuid)
                    .if_not_exists()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_order_change_logs_order_uuid")
                    .table(OrderChangeLogs::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(Table::drop().table(OrderChangeLogs::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum OrderChangeLogs {
    Table,
    Uuid,
    OrderUuid,
    Action,
    OperatorUuid,
    BeforeData,
    AfterData,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Orders {
    Table,
    Uuid,
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm_migration::sea_orm::DbBackend;

    #[test]
    fn create_table_sql_includes_order_change_logs_structure() {
        let sql = DbBackend::Postgres
            .build(
                &Table::create()
                    .table(OrderChangeLogs::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(OrderChangeLogs::Uuid)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(OrderChangeLogs::OrderUuid).uuid().not_null())
                    .col(ColumnDef::new(OrderChangeLogs::Action).string().not_null())
                    .col(ColumnDef::new(OrderChangeLogs::OperatorUuid).uuid().null())
                    .col(ColumnDef::new(OrderChangeLogs::BeforeData).json_binary().null())
                    .col(ColumnDef::new(OrderChangeLogs::AfterData).json_binary().null())
                    .col(
                        ColumnDef::new(OrderChangeLogs::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_order_change_logs_order")
                            .from(OrderChangeLogs::Table, OrderChangeLogs::OrderUuid)
                            .to(Orders::Table, Orders::Uuid)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .to_string();

        assert!(sql.contains(r#"CREATE TABLE IF NOT EXISTS "order_change_logs""#));
        assert!(sql.contains(r#""before_data" jsonb"#));
        assert!(sql.contains(r#""after_data" jsonb"#));
        assert!(sql.contains(r#"CONSTRAINT "fk_order_change_logs_order""#));
    }
}
