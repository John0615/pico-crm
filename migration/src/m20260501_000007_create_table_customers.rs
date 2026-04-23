use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Customers::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Customers::Uuid)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Customers::MerchantId).uuid().null())
                    .col(ColumnDef::new(Customers::UserName).string().not_null())
                    .col(ColumnDef::new(Customers::PhoneNumber).string().not_null())
                    .col(ColumnDef::new(Customers::Address).text().null())
                    .col(ColumnDef::new(Customers::Community).string().null())
                    .col(ColumnDef::new(Customers::Building).string().null())
                    .col(ColumnDef::new(Customers::HouseAreaSqm).integer().null())
                    .col(ColumnDef::new(Customers::ServiceNeed).text().null())
                    .col(
                        ColumnDef::new(Customers::Tags)
                            .json_binary()
                            .not_null()
                            .default(Expr::val("[]")),
                    )
                    .col(
                        ColumnDef::new(Customers::LastServiceAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Customers::FollowUpStatus)
                            .string()
                            .not_null()
                            .default("new"),
                    )
                    .col(ColumnDef::new(Customers::CreatorUuid).uuid().not_null())
                    .col(
                        ColumnDef::new(Customers::InsertedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Customers::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_customers_merchant")
                            .from(Customers::Table, Customers::MerchantId)
                            .to(Merchant::Table, Merchant::Uuid)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_customers_creator")
                            .from(Customers::Table, Customers::CreatorUuid)
                            .to(Users::Table, Users::Uuid)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_customers_merchant_id")
                    .table(Customers::Table)
                    .col(Customers::MerchantId)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_customers_merchant_phone_unique")
                    .table(Customers::Table)
                    .col(Customers::MerchantId)
                    .col(Customers::PhoneNumber)
                    .unique()
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_customers_merchant_follow_up_status")
                    .table(Customers::Table)
                    .col(Customers::MerchantId)
                    .col(Customers::FollowUpStatus)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Customers::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Customers {
    Table,
    Uuid,
    MerchantId,
    UserName,
    PhoneNumber,
    Address,
    Community,
    Building,
    HouseAreaSqm,
    ServiceNeed,
    Tags,
    LastServiceAt,
    FollowUpStatus,
    CreatorUuid,
    InsertedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Uuid,
}

#[derive(DeriveIden)]
enum Merchant {
    Table,
    Uuid,
}
