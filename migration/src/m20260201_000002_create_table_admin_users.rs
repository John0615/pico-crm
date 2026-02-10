use sea_orm_migration::prelude::*;

const PUBLIC_SCHEMA: &str = "public";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table((Alias::new(PUBLIC_SCHEMA), AdminUser::Table))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AdminUser::Uuid)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(
                        ColumnDef::new(AdminUser::UserName)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(AdminUser::Password)
                            .string()
                            .string_len(255)
                            .not_null(),
                    )
                    .col(ColumnDef::new(AdminUser::Email).string().null().unique_key())
                    .col(ColumnDef::new(AdminUser::PhoneNumber).string().null().unique_key())
                    .col(
                        ColumnDef::new(AdminUser::Role)
                            .string()
                            .not_null()
                            .default("admin"),
                    )
                    .col(
                        ColumnDef::new(AdminUser::Status)
                            .string()
                            .not_null()
                            .default("active"),
                    )
                    .col(ColumnDef::new(AdminUser::LastLoginAt).timestamp_with_time_zone().null())
                    .col(
                        ColumnDef::new(AdminUser::InsertedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(AdminUser::UpdatedAt)
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
                    .table((Alias::new(PUBLIC_SCHEMA), AdminUser::Table))
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum AdminUser {
    Table,
    Uuid,
    UserName,
    Password,
    Email,
    PhoneNumber,
    Role,
    Status,
    LastLoginAt,
    InsertedAt,
    UpdatedAt,
}
