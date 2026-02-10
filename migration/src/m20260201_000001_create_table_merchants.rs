use sea_orm_migration::prelude::*;

const PUBLIC_SCHEMA: &str = "public";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table((Alias::new(PUBLIC_SCHEMA), Merchant::Table))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Merchant::Uuid)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(Merchant::Name).string().not_null())
                    .col(ColumnDef::new(Merchant::ShortName).string().null())
                    .col(
                        ColumnDef::new(Merchant::SchemaName)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Merchant::ContactName).string().not_null())
                    .col(
                        ColumnDef::new(Merchant::ContactPhone)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Merchant::MerchantType).string().null())
                    .col(ColumnDef::new(Merchant::PlanType).string().null())
                    .col(
                        ColumnDef::new(Merchant::Status)
                            .string()
                            .not_null()
                            .default("active"),
                    )
                    .col(ColumnDef::new(Merchant::TrialEndAt).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(Merchant::ExpiredAt).timestamp_with_time_zone().null())
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
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table((Alias::new(PUBLIC_SCHEMA), Merchant::Table))
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Merchant {
    Table,
    Uuid,
    Name,
    ShortName,
    SchemaName,
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
