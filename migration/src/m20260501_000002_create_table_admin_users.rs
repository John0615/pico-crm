use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AdminUsers::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AdminUsers::Uuid)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(AdminUsers::UserName).string().not_null())
                    .col(ColumnDef::new(AdminUsers::Password).string().not_null())
                    .col(ColumnDef::new(AdminUsers::Email).string().null())
                    .col(ColumnDef::new(AdminUsers::PhoneNumber).string().null())
                    .col(
                        ColumnDef::new(AdminUsers::Role)
                            .string()
                            .not_null()
                            .default("admin"),
                    )
                    .col(
                        ColumnDef::new(AdminUsers::Status)
                            .string()
                            .not_null()
                            .default("active"),
                    )
                    .col(
                        ColumnDef::new(AdminUsers::LastLoginAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AdminUsers::InsertedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(AdminUsers::UpdatedAt)
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
                    .name("idx_admin_users_username_unique")
                    .table(AdminUsers::Table)
                    .col(AdminUsers::UserName)
                    .unique()
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_admin_users_email_unique")
                    .table(AdminUsers::Table)
                    .col(AdminUsers::Email)
                    .unique()
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_admin_users_phone_unique")
                    .table(AdminUsers::Table)
                    .col(AdminUsers::PhoneNumber)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AdminUsers::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum AdminUsers {
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
