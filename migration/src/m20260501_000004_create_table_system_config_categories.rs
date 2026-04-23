use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(SystemConfigCategories::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SystemConfigCategories::Code)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(SystemConfigCategories::Name)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SystemConfigCategories::Description)
                            .text()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(SystemConfigCategories::SortOrder)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(SystemConfigCategories::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(SystemConfigCategories::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(SystemConfigCategories::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(SystemConfigCategories::Table)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum SystemConfigCategories {
    Table,
    Code,
    Name,
    Description,
    SortOrder,
    IsActive,
    CreatedAt,
    UpdatedAt,
}
