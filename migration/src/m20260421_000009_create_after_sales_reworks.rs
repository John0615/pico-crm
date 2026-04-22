use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AfterSalesReworks::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AfterSalesReworks::Uuid)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(
                        ColumnDef::new(AfterSalesReworks::CaseUuid)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AfterSalesReworks::AssignedUserUuid)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AfterSalesReworks::ScheduledStartAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AfterSalesReworks::ScheduledEndAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(ColumnDef::new(AfterSalesReworks::Note).text().null())
                    .col(
                        ColumnDef::new(AfterSalesReworks::Status)
                            .string()
                            .not_null()
                            .default("planned"),
                    )
                    .col(
                        ColumnDef::new(AfterSalesReworks::InsertedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_after_sales_reworks_case")
                            .from(AfterSalesReworks::Table, AfterSalesReworks::CaseUuid)
                            .to(AfterSalesCases::Table, AfterSalesCases::Uuid)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_after_sales_reworks_assigned_user")
                            .from(
                                AfterSalesReworks::Table,
                                AfterSalesReworks::AssignedUserUuid,
                            )
                            .to(Users::Table, Users::Uuid)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_after_sales_reworks_case_uuid")
                    .table(AfterSalesReworks::Table)
                    .col(AfterSalesReworks::CaseUuid)
                    .if_not_exists()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_after_sales_reworks_case_uuid")
                    .table(AfterSalesReworks::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(Table::drop().table(AfterSalesReworks::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum AfterSalesReworks {
    Table,
    Uuid,
    CaseUuid,
    AssignedUserUuid,
    ScheduledStartAt,
    ScheduledEndAt,
    Note,
    Status,
    InsertedAt,
}

#[derive(DeriveIden)]
enum AfterSalesCases {
    Table,
    Uuid,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Uuid,
}
