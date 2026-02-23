use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::Statement;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(ServiceRequests::Table)
                    .add_column_if_not_exists(ColumnDef::new(ServiceRequests::CreatorUuid).uuid().null())
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute(Statement::from_string(
                manager.get_database_backend(),
                "UPDATE service_requests SET creator_uuid = (SELECT uuid FROM users ORDER BY inserted_at ASC LIMIT 1) WHERE creator_uuid IS NULL",
            ))
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(ServiceRequests::Table)
                    .modify_column(ColumnDef::new(ServiceRequests::CreatorUuid).uuid().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_service_requests_creator")
                    .from(ServiceRequests::Table, ServiceRequests::CreatorUuid)
                    .to(Users::Table, Users::Uuid)
                    .on_delete(ForeignKeyAction::Restrict)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_service_requests_creator")
                    .table(ServiceRequests::Table)
                    .col(ServiceRequests::CreatorUuid)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_service_requests_creator")
                    .table(ServiceRequests::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_service_requests_creator")
                    .table(ServiceRequests::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(ServiceRequests::Table)
                    .drop_column(ServiceRequests::CreatorUuid)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum ServiceRequests {
    Table,
    CreatorUuid,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Uuid,
}
