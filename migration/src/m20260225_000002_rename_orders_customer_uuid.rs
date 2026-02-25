use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_orders_contact")
                    .table(Orders::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_orders_contact")
                    .table(Orders::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_orders_customer")
                    .table(Orders::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Orders::Table)
                    .rename_column(Orders::CustomerId, Orders::CustomerUuid)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Orders::Table)
                    .drop_column(Orders::ContactUuid)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_orders_customer")
                    .from(Orders::Table, Orders::CustomerUuid)
                    .to(Customers::Table, Customers::Uuid)
                    .on_delete(ForeignKeyAction::Restrict)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_orders_customer")
                    .table(Orders::Table)
                    .col(Orders::CustomerUuid)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_orders_customer")
                    .table(Orders::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_orders_customer")
                    .table(Orders::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Orders::Table)
                    .rename_column(Orders::CustomerUuid, Orders::CustomerId)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Orders::Table)
                    .add_column_if_not_exists(ColumnDef::new(Orders::ContactUuid).uuid().null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_orders_contact")
                    .from(Orders::Table, Orders::ContactUuid)
                    .to(Customers::Table, Customers::Uuid)
                    .on_delete(ForeignKeyAction::Restrict)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_orders_contact")
                    .table(Orders::Table)
                    .col(Orders::ContactUuid)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_orders_customer")
                    .from(Orders::Table, Orders::CustomerId)
                    .to(Customers::Table, Customers::Uuid)
                    .on_delete(ForeignKeyAction::Restrict)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Orders {
    Table,
    CustomerId,
    CustomerUuid,
    ContactUuid,
}

#[derive(DeriveIden)]
enum Customers {
    Table,
    Uuid,
}
