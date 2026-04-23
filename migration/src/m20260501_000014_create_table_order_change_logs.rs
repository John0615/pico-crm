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
                            .primary_key(),
                    )
                    .col(ColumnDef::new(OrderChangeLogs::MerchantId).uuid().null())
                    .col(ColumnDef::new(OrderChangeLogs::OrderUuid).uuid().not_null())
                    .col(ColumnDef::new(OrderChangeLogs::Action).string().not_null())
                    .col(ColumnDef::new(OrderChangeLogs::OperatorUuid).uuid().null())
                    .col(
                        ColumnDef::new(OrderChangeLogs::BeforeData)
                            .json_binary()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(OrderChangeLogs::AfterData)
                            .json_binary()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(OrderChangeLogs::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_order_change_logs_merchant")
                            .from(OrderChangeLogs::Table, OrderChangeLogs::MerchantId)
                            .to(Merchant::Table, Merchant::Uuid)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_order_change_logs_order")
                            .from(OrderChangeLogs::Table, OrderChangeLogs::OrderUuid)
                            .to(Orders::Table, Orders::Uuid)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_order_change_logs_operator")
                            .from(OrderChangeLogs::Table, OrderChangeLogs::OperatorUuid)
                            .to(Users::Table, Users::Uuid)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_order_change_logs_order_created_at")
                    .table(OrderChangeLogs::Table)
                    .col(OrderChangeLogs::OrderUuid)
                    .col(OrderChangeLogs::CreatedAt)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(OrderChangeLogs::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum OrderChangeLogs {
    Table,
    Uuid,
    MerchantId,
    OrderUuid,
    Action,
    OperatorUuid,
    BeforeData,
    AfterData,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Merchant {
    Table,
    Uuid,
}

#[derive(DeriveIden)]
enum Orders {
    Table,
    Uuid,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Uuid,
}
