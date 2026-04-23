use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Schedules::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Schedules::Uuid)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Schedules::MerchantId).uuid().null())
                    .col(ColumnDef::new(Schedules::OrderUuid).uuid().not_null())
                    .col(
                        ColumnDef::new(Schedules::AssignedUserUuid)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Schedules::StartAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Schedules::EndAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Schedules::Status)
                            .string()
                            .not_null()
                            .default("planned"),
                    )
                    .col(ColumnDef::new(Schedules::Notes).text().null())
                    .col(
                        ColumnDef::new(Schedules::InsertedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Schedules::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Schedules::EventId)
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_schedules_merchant")
                            .from(Schedules::Table, Schedules::MerchantId)
                            .to(Merchant::Table, Merchant::Uuid)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_schedules_order")
                            .from(Schedules::Table, Schedules::OrderUuid)
                            .to(Orders::Table, Orders::Uuid)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_schedules_user")
                            .from(Schedules::Table, Schedules::AssignedUserUuid)
                            .to(Users::Table, Users::Uuid)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_schedules_order_unique")
                    .table(Schedules::Table)
                    .col(Schedules::OrderUuid)
                    .unique()
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_schedules_merchant_user_time")
                    .table(Schedules::Table)
                    .col(Schedules::MerchantId)
                    .col(Schedules::AssignedUserUuid)
                    .col(Schedules::StartAt)
                    .col(Schedules::EndAt)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Schedules::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Schedules {
    Table,
    Uuid,
    MerchantId,
    OrderUuid,
    AssignedUserUuid,
    StartAt,
    EndAt,
    Status,
    Notes,
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
enum Orders {
    Table,
    Uuid,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Uuid,
}
