use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(ServiceRequests::Table)
                    .add_column_if_not_exists(
                        ColumnDef::new(ServiceRequests::EventId)
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(ServiceRequests::Table)
                    .drop_column(ServiceRequests::EventId)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum ServiceRequests {
    Table,
    EventId,
}
