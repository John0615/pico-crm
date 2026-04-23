use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(SystemConfigItems::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SystemConfigItems::Uuid)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(SystemConfigItems::CategoryCode)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(SystemConfigItems::Key).string().not_null())
                    .col(ColumnDef::new(SystemConfigItems::Label).string().not_null())
                    .col(ColumnDef::new(SystemConfigItems::Description).text().null())
                    .col(
                        ColumnDef::new(SystemConfigItems::ValueType)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SystemConfigItems::DefaultValue)
                            .json_binary()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SystemConfigItems::Value)
                            .json_binary()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(SystemConfigItems::Validation)
                            .json_binary()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(SystemConfigItems::UiSchema)
                            .json_binary()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(SystemConfigItems::IsRequired)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(SystemConfigItems::IsEditable)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(SystemConfigItems::IsSensitive)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(SystemConfigItems::SortOrder)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(SystemConfigItems::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(SystemConfigItems::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_system_config_items_category")
                            .from(SystemConfigItems::Table, SystemConfigItems::CategoryCode)
                            .to(SystemConfigCategories::Table, SystemConfigCategories::Code)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_system_config_items_key_unique")
                    .table(SystemConfigItems::Table)
                    .col(SystemConfigItems::Key)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(SystemConfigItems::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum SystemConfigItems {
    Table,
    Uuid,
    CategoryCode,
    Key,
    Label,
    Description,
    ValueType,
    DefaultValue,
    Value,
    Validation,
    UiSchema,
    IsRequired,
    IsEditable,
    IsSensitive,
    SortOrder,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum SystemConfigCategories {
    Table,
    Code,
}
