use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(OrderFeedback::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(OrderFeedback::Uuid)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(OrderFeedback::MerchantId).uuid().null())
                    .col(ColumnDef::new(OrderFeedback::OrderId).uuid().not_null())
                    .col(ColumnDef::new(OrderFeedback::WorkerId).uuid().null())
                    .col(ColumnDef::new(OrderFeedback::UserUuid).uuid().null())
                    .col(ColumnDef::new(OrderFeedback::Rating).integer().null())
                    .col(ColumnDef::new(OrderFeedback::Content).text().null())
                    .col(
                        ColumnDef::new(OrderFeedback::InsertedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(OrderFeedback::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_order_feedback_merchant")
                            .from(OrderFeedback::Table, OrderFeedback::MerchantId)
                            .to(Merchant::Table, Merchant::Uuid)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_order_feedback_order")
                            .from(OrderFeedback::Table, OrderFeedback::OrderId)
                            .to(Orders::Table, Orders::Uuid)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_order_feedback_user")
                            .from(OrderFeedback::Table, OrderFeedback::UserUuid)
                            .to(Users::Table, Users::Uuid)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_order_feedback_order")
                    .table(OrderFeedback::Table)
                    .col(OrderFeedback::OrderId)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_order_feedback_merchant_order_user_unique")
                    .table(OrderFeedback::Table)
                    .col(OrderFeedback::MerchantId)
                    .col(OrderFeedback::OrderId)
                    .col(OrderFeedback::UserUuid)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(OrderFeedback::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum OrderFeedback {
    Table,
    Uuid,
    MerchantId,
    OrderId,
    WorkerId,
    UserUuid,
    Rating,
    Content,
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
