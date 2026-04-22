use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_index(
                Index::create()
                    .name("ux_order_feedback_order_user")
                    .table(OrderFeedback::Table)
                    .col(OrderFeedback::OrderId)
                    .col(OrderFeedback::UserUuid)
                    .unique()
                    .if_not_exists()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("ux_order_feedback_order_user")
                    .table(OrderFeedback::Table)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum OrderFeedback {
    Table,
    OrderId,
    UserUuid,
}
