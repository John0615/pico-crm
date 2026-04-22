use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ServiceCatalogs::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ServiceCatalogs::Uuid)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(ServiceCatalogs::Name).string().not_null())
                    .col(ColumnDef::new(ServiceCatalogs::Description).text().null())
                    .col(
                        ColumnDef::new(ServiceCatalogs::BasePriceCents)
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(ServiceCatalogs::DefaultDurationMinutes)
                            .integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ServiceCatalogs::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(ServiceCatalogs::SortOrder)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(ServiceCatalogs::InsertedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(ServiceCatalogs::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_service_catalogs_is_active")
                    .table(ServiceCatalogs::Table)
                    .col(ServiceCatalogs::IsActive)
                    .if_not_exists()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_service_catalogs_is_active")
                    .table(ServiceCatalogs::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(ServiceCatalogs::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum ServiceCatalogs {
    Table,
    Uuid,
    Name,
    Description,
    BasePriceCents,
    DefaultDurationMinutes,
    IsActive,
    SortOrder,
    InsertedAt,
    UpdatedAt,
}
