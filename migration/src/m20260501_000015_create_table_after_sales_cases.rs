use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AfterSalesCases::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AfterSalesCases::Uuid)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(AfterSalesCases::MerchantId).uuid().null())
                    .col(ColumnDef::new(AfterSalesCases::OrderUuid).uuid().not_null())
                    .col(ColumnDef::new(AfterSalesCases::OperatorUuid).uuid().null())
                    .col(
                        ColumnDef::new(AfterSalesCases::CaseType)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AfterSalesCases::Description)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AfterSalesCases::Status)
                            .string()
                            .not_null()
                            .default("open"),
                    )
                    .col(
                        ColumnDef::new(AfterSalesCases::RefundAmountCents)
                            .big_integer()
                            .null(),
                    )
                    .col(ColumnDef::new(AfterSalesCases::RefundReason).text().null())
                    .col(
                        ColumnDef::new(AfterSalesCases::InsertedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(AfterSalesCases::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_after_sales_cases_merchant")
                            .from(AfterSalesCases::Table, AfterSalesCases::MerchantId)
                            .to(Merchant::Table, Merchant::Uuid)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_after_sales_cases_order")
                            .from(AfterSalesCases::Table, AfterSalesCases::OrderUuid)
                            .to(Orders::Table, Orders::Uuid)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_after_sales_cases_operator")
                            .from(AfterSalesCases::Table, AfterSalesCases::OperatorUuid)
                            .to(Users::Table, Users::Uuid)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_after_sales_cases_order")
                    .table(AfterSalesCases::Table)
                    .col(AfterSalesCases::OrderUuid)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AfterSalesCases::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum AfterSalesCases {
    Table,
    Uuid,
    MerchantId,
    OrderUuid,
    OperatorUuid,
    CaseType,
    Description,
    Status,
    RefundAmountCents,
    RefundReason,
    InsertedAt,
    UpdatedAt,
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
