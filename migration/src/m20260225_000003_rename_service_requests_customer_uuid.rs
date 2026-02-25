use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_service_requests_contact")
                    .table(ServiceRequests::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_service_requests_contact")
                    .table(ServiceRequests::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(ServiceRequests::Table)
                    .rename_column(ServiceRequests::ContactUuid, ServiceRequests::CustomerUuid)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_service_requests_customer")
                    .from(ServiceRequests::Table, ServiceRequests::CustomerUuid)
                    .to(Customers::Table, Customers::Uuid)
                    .on_delete(ForeignKeyAction::Restrict)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_service_requests_customer")
                    .table(ServiceRequests::Table)
                    .col(ServiceRequests::CustomerUuid)
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
                    .name("idx_service_requests_customer")
                    .table(ServiceRequests::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_service_requests_customer")
                    .table(ServiceRequests::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(ServiceRequests::Table)
                    .rename_column(ServiceRequests::CustomerUuid, ServiceRequests::ContactUuid)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_service_requests_contact")
                    .from(ServiceRequests::Table, ServiceRequests::ContactUuid)
                    .to(Customers::Table, Customers::Uuid)
                    .on_delete(ForeignKeyAction::Restrict)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_service_requests_contact")
                    .table(ServiceRequests::Table)
                    .col(ServiceRequests::ContactUuid)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum ServiceRequests {
    Table,
    ContactUuid,
    CustomerUuid,
}

#[derive(DeriveIden)]
enum Customers {
    Table,
    Uuid,
}
