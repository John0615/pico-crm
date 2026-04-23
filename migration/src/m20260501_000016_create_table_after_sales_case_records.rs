use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AfterSalesCaseRecords::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AfterSalesCaseRecords::Uuid)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(AfterSalesCaseRecords::MerchantId)
                            .uuid()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AfterSalesCaseRecords::CaseUuid)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AfterSalesCaseRecords::OperatorUuid)
                            .uuid()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AfterSalesCaseRecords::Content)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AfterSalesCaseRecords::Status)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AfterSalesCaseRecords::InsertedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_after_sales_case_records_merchant")
                            .from(
                                AfterSalesCaseRecords::Table,
                                AfterSalesCaseRecords::MerchantId,
                            )
                            .to(Merchant::Table, Merchant::Uuid)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_after_sales_case_records_case")
                            .from(
                                AfterSalesCaseRecords::Table,
                                AfterSalesCaseRecords::CaseUuid,
                            )
                            .to(AfterSalesCases::Table, AfterSalesCases::Uuid)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_after_sales_case_records_operator")
                            .from(
                                AfterSalesCaseRecords::Table,
                                AfterSalesCaseRecords::OperatorUuid,
                            )
                            .to(Users::Table, Users::Uuid)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_after_sales_case_records_case")
                    .table(AfterSalesCaseRecords::Table)
                    .col(AfterSalesCaseRecords::CaseUuid)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AfterSalesCaseRecords::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum AfterSalesCaseRecords {
    Table,
    Uuid,
    MerchantId,
    CaseUuid,
    OperatorUuid,
    Content,
    Status,
    InsertedAt,
}

#[derive(DeriveIden)]
enum Merchant {
    Table,
    Uuid,
}

#[derive(DeriveIden)]
enum AfterSalesCases {
    Table,
    Uuid,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Uuid,
}
