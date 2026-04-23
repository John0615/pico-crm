use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ServiceRequests::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ServiceRequests::Uuid)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ServiceRequests::MerchantId).uuid().null())
                    .col(
                        ColumnDef::new(ServiceRequests::CustomerUuid)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ServiceRequests::CreatorUuid)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ServiceRequests::ServiceCatalogUuid)
                            .uuid()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ServiceRequests::ServiceContent)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ServiceRequests::AppointmentStartAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ServiceRequests::AppointmentEndAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ServiceRequests::Status)
                            .string()
                            .not_null()
                            .default("new"),
                    )
                    .col(
                        ColumnDef::new(ServiceRequests::Source)
                            .string()
                            .not_null()
                            .default("manual"),
                    )
                    .col(ColumnDef::new(ServiceRequests::Notes).text().null())
                    .col(
                        ColumnDef::new(ServiceRequests::InsertedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(ServiceRequests::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(ServiceRequests::EventId)
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_service_requests_merchant")
                            .from(ServiceRequests::Table, ServiceRequests::MerchantId)
                            .to(Merchant::Table, Merchant::Uuid)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_service_requests_customer")
                            .from(ServiceRequests::Table, ServiceRequests::CustomerUuid)
                            .to(Customers::Table, Customers::Uuid)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_service_requests_creator")
                            .from(ServiceRequests::Table, ServiceRequests::CreatorUuid)
                            .to(Users::Table, Users::Uuid)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_service_requests_catalog")
                            .from(ServiceRequests::Table, ServiceRequests::ServiceCatalogUuid)
                            .to(ServiceCatalogs::Table, ServiceCatalogs::Uuid)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_service_requests_merchant_status")
                    .table(ServiceRequests::Table)
                    .col(ServiceRequests::MerchantId)
                    .col(ServiceRequests::Status)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_service_requests_merchant_appointment")
                    .table(ServiceRequests::Table)
                    .col(ServiceRequests::MerchantId)
                    .col(ServiceRequests::AppointmentStartAt)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ServiceRequests::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum ServiceRequests {
    Table,
    Uuid,
    MerchantId,
    CustomerUuid,
    CreatorUuid,
    ServiceCatalogUuid,
    ServiceContent,
    AppointmentStartAt,
    AppointmentEndAt,
    Status,
    Source,
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
enum Customers {
    Table,
    Uuid,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Uuid,
}

#[derive(DeriveIden)]
enum ServiceCatalogs {
    Table,
    Uuid,
}
