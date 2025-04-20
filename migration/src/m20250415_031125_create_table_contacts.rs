use sea_orm_migration::{prelude::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Contacts::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Contacts::ContactUuid).uuid().not_null().default(Expr::cust("gen_random_uuid()")))
                    .col(ColumnDef::new(Contacts::UserName).string().not_null())
                    .col(ColumnDef::new(Contacts::Company).string().not_null())
                    .col(ColumnDef::new(Contacts::Position).string().not_null())
                    .col(ColumnDef::new(Contacts::PhoneNumber).string().not_null())
                    .col(ColumnDef::new(Contacts::Email).string().not_null())
                    .col(ColumnDef::new(Contacts::LastContact).date_time().not_null())
                    .col(ColumnDef::new(Contacts::ValueLevel).integer().not_null())
                    .col(ColumnDef::new(Contacts::CreatorUuid).uuid().not_null())
                    .col(ColumnDef::new(Contacts::Status).integer().not_null())
                    .col(
                        ColumnDef::new(Contacts::InsertedAt)
                        .date_time()
                        .not_null()
                    )
                    .col(
                        ColumnDef::new(Contacts::UpdatedAt)
                        .date_time()
                        .not_null()
                    )
                    .primary_key(
                        Index::create()
                            .name("ix_contact_contact_uuid_index")
                            .col(Contacts::ContactUuid),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Contacts::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Contacts {
    Table,
    ContactUuid,
    UserName,
    Company,
    Position,
    PhoneNumber,
    Email,
    Status,
    LastContact,
    ValueLevel,
    CreatorUuid,
    InsertedAt,
    UpdatedAt,
}
