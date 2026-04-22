use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(OrderFeedback::Table)
                    .add_column_if_not_exists(ColumnDef::new(OrderFeedback::UserUuid).uuid().null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_order_feedback_user")
                    .from(OrderFeedback::Table, OrderFeedback::UserUuid)
                    .to(Users::Table, Users::Uuid)
                    .on_delete(ForeignKeyAction::SetNull)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_order_feedback_order_id")
                    .table(OrderFeedback::Table)
                    .col(OrderFeedback::OrderId)
                    .if_not_exists()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_order_feedback_order_id")
                    .table(OrderFeedback::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_order_feedback_user")
                    .table(OrderFeedback::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(OrderFeedback::Table)
                    .drop_column(OrderFeedback::UserUuid)
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

#[derive(DeriveIden)]
enum Users {
    Table,
    Uuid,
}
