use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Users::Uuid)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(
                        ColumnDef::new(Users::UserName)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(Users::Password)
                            .string()
                            .string_len(255)
                            .not_null(),
                    ) // 密码长度限制
                    .col(ColumnDef::new(Users::Email).string().null().unique_key())
                    .col(
                        ColumnDef::new(Users::PhoneNumber)
                            .string()
                            .null()
                            .unique_key(),
                    ) // 手机号唯一
                    .col(ColumnDef::new(Users::IsAdmin).boolean().default(false)) // 是否管理员
                    .col(
                        ColumnDef::new(Users::Status)
                            .string_len(10)
                            .not_null()
                            .default("active"), // 用户状态
                    )
                    .col(ColumnDef::new(Users::AvatarUrl).text().null()) // 头像 URL（可选）
                    .col(ColumnDef::new(Users::LastLoginAt).date_time().null()) // 最后登录时间
                    .col(ColumnDef::new(Users::EmailVerifiedAt).date_time().null()) // 邮箱验证时间
                    .col(
                        ColumnDef::new(Users::InsertedAt).date_time().not_null(), // 使用 date_time
                    )
                    .col(
                        ColumnDef::new(Users::UpdatedAt).date_time().not_null(), // 使用 date_time
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Uuid,
    UserName,
    Password,
    Email,
    PhoneNumber,
    IsAdmin,
    Status,
    AvatarUrl,
    LastLoginAt,
    EmailVerifiedAt,
    InsertedAt,
    UpdatedAt,
}
