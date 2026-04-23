use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ContactFollowRecords::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ContactFollowRecords::Uuid)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ContactFollowRecords::MerchantId)
                            .uuid()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ContactFollowRecords::ContactUuid)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ContactFollowRecords::OperatorUuid)
                            .uuid()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ContactFollowRecords::Content)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ContactFollowRecords::NextFollowUpAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ContactFollowRecords::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_contact_follow_records_merchant")
                            .from(
                                ContactFollowRecords::Table,
                                ContactFollowRecords::MerchantId,
                            )
                            .to(Merchant::Table, Merchant::Uuid)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_contact_follow_records_contact")
                            .from(
                                ContactFollowRecords::Table,
                                ContactFollowRecords::ContactUuid,
                            )
                            .to(Customers::Table, Customers::Uuid)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_contact_follow_records_operator")
                            .from(
                                ContactFollowRecords::Table,
                                ContactFollowRecords::OperatorUuid,
                            )
                            .to(Users::Table, Users::Uuid)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_contact_follow_records_contact")
                    .table(ContactFollowRecords::Table)
                    .col(ContactFollowRecords::ContactUuid)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ContactFollowRecords::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum ContactFollowRecords {
    Table,
    Uuid,
    MerchantId,
    ContactUuid,
    OperatorUuid,
    Content,
    NextFollowUpAt,
    CreatedAt,
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
