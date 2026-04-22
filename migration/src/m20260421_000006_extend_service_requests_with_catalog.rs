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
                        ColumnDef::new(ServiceRequests::ServiceCatalogUuid)
                            .uuid()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_service_requests_service_catalog")
                    .from(ServiceRequests::Table, ServiceRequests::ServiceCatalogUuid)
                    .to(ServiceCatalogs::Table, ServiceCatalogs::Uuid)
                    .on_delete(ForeignKeyAction::SetNull)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_service_requests_service_catalog")
                    .table(ServiceRequests::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(ServiceRequests::Table)
                    .drop_column(ServiceRequests::ServiceCatalogUuid)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum ServiceRequests {
    Table,
    ServiceCatalogUuid,
}

#[derive(DeriveIden)]
enum ServiceCatalogs {
    Table,
    Uuid,
}
