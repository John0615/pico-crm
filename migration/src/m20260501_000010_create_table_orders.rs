use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Orders::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Orders::Uuid).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Orders::MerchantId).uuid().null())
                    .col(ColumnDef::new(Orders::CustomerUuid).uuid().null())
                    .col(
                        ColumnDef::new(Orders::Status)
                            .string()
                            .not_null()
                            .default("pending"),
                    )
                    .col(ColumnDef::new(Orders::CancellationReason).text().null())
                    .col(
                        ColumnDef::new(Orders::CompletedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Orders::AmountCents)
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(ColumnDef::new(Orders::PaidAmountCents).big_integer().null())
                    .col(ColumnDef::new(Orders::PaymentMethod).string().null())
                    .col(
                        ColumnDef::new(Orders::PaidAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(ColumnDef::new(Orders::Notes).text().null())
                    .col(ColumnDef::new(Orders::RequestId).uuid().null())
                    .col(
                        ColumnDef::new(Orders::ScheduledStartAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Orders::ScheduledEndAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(ColumnDef::new(Orders::DispatchNote).text().null())
                    .col(
                        ColumnDef::new(Orders::SettlementStatus)
                            .string()
                            .not_null()
                            .default("unsettled"),
                    )
                    .col(ColumnDef::new(Orders::SettlementNote).text().null())
                    .col(
                        ColumnDef::new(Orders::InsertedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Orders::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Orders::EventId)
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_orders_merchant")
                            .from(Orders::Table, Orders::MerchantId)
                            .to(Merchant::Table, Merchant::Uuid)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_orders_customer")
                            .from(Orders::Table, Orders::CustomerUuid)
                            .to(Customers::Table, Customers::Uuid)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_orders_request")
                            .from(Orders::Table, Orders::RequestId)
                            .to(ServiceRequests::Table, ServiceRequests::Uuid)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_orders_merchant_status")
                    .table(Orders::Table)
                    .col(Orders::MerchantId)
                    .col(Orders::Status)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_orders_merchant_scheduled_start")
                    .table(Orders::Table)
                    .col(Orders::MerchantId)
                    .col(Orders::ScheduledStartAt)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_orders_request_id")
                    .table(Orders::Table)
                    .col(Orders::RequestId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Orders::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Orders {
    Table,
    Uuid,
    MerchantId,
    CustomerUuid,
    Status,
    CancellationReason,
    CompletedAt,
    AmountCents,
    PaidAmountCents,
    PaymentMethod,
    PaidAt,
    Notes,
    RequestId,
    ScheduledStartAt,
    ScheduledEndAt,
    DispatchNote,
    SettlementStatus,
    SettlementNote,
    InsertedAt,
    UpdatedAt,
    EventId,
}

#[derive(DeriveIden)]
enum Merchant {
    Table,
    Uuid,
}

#[derive(DeriveIden)]
enum Customers {
    Table,
    Uuid,
}

#[derive(DeriveIden)]
enum ServiceRequests {
    Table,
    Uuid,
}
