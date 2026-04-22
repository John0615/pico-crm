use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(AfterSalesCases::Table)
                    .add_column_if_not_exists(
                        ColumnDef::new(AfterSalesCases::RefundAmountCents)
                            .big_integer()
                            .null(),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(AfterSalesCases::RefundReason).text().null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(AfterSalesCases::Table)
                    .drop_column(AfterSalesCases::RefundReason)
                    .drop_column(AfterSalesCases::RefundAmountCents)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum AfterSalesCases {
    Table,
    RefundAmountCents,
    RefundReason,
}
