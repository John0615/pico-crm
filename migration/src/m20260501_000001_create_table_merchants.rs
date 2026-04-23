use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Merchant::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Merchant::Uuid)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Merchant::Name).string().not_null())
                    .col(ColumnDef::new(Merchant::ShortName).string().null())
                    .col(ColumnDef::new(Merchant::ContactName).string().not_null())
                    .col(ColumnDef::new(Merchant::ContactPhone).string().not_null())
                    .col(ColumnDef::new(Merchant::MerchantType).string().null())
                    .col(ColumnDef::new(Merchant::PlanType).string().null())
                    .col(
                        ColumnDef::new(Merchant::Status)
                            .string()
                            .not_null()
                            .default("active"),
                    )
                    .col(
                        ColumnDef::new(Merchant::TrialEndAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Merchant::ExpiredAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Merchant::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Merchant::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_merchant_contact_phone_unique")
                    .table(Merchant::Table)
                    .col(Merchant::ContactPhone)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Merchant::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Merchant {
    Table,
    Uuid,
    Name,
    ShortName,
    ContactName,
    ContactPhone,
    MerchantType,
    PlanType,
    Status,
    TrialEndAt,
    ExpiredAt,
    CreatedAt,
    UpdatedAt,
}
