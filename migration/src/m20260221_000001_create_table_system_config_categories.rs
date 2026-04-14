use sea_orm_migration::prelude::*;

const PUBLIC_SCHEMA: &str = "public";
const SYSTEM_CONFIG_CATEGORIES: &str = "system_config_categories";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table((
                        Alias::new(PUBLIC_SCHEMA),
                        Alias::new(SYSTEM_CONFIG_CATEGORIES),
                    ))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SystemConfigCategory::Code)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(SystemConfigCategory::Name)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SystemConfigCategory::Description)
                            .string()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(SystemConfigCategory::SortOrder)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(SystemConfigCategory::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(SystemConfigCategory::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(SystemConfigCategory::UpdatedAt)
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
                    .table((
                        Alias::new(PUBLIC_SCHEMA),
                        Alias::new(SYSTEM_CONFIG_CATEGORIES),
                    ))
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum SystemConfigCategory {
    Code,
    Name,
    Description,
    SortOrder,
    IsActive,
    CreatedAt,
    UpdatedAt,
}
