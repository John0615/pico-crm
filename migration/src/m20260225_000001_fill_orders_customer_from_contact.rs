use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::{ConnectionTrait, Statement};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop foreign keys that reference contacts/customers before renaming.
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_service_requests_contact")
                    .table(ServiceRequests::Table)
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

        let backend = manager.get_database_backend();

        // Archive the legacy customers table and promote contacts to customers.
        manager
            .get_connection()
            .execute(Statement::from_string(
                backend,
                "ALTER TABLE IF EXISTS customers RENAME TO customers_legacy".to_string(),
            ))
            .await?;
        manager
            .get_connection()
            .execute(Statement::from_string(
                backend,
                "ALTER TABLE IF EXISTS contacts RENAME TO customers".to_string(),
            ))
            .await?;
        manager
            .get_connection()
            .execute(Statement::from_string(
                backend,
                "ALTER TABLE customers RENAME COLUMN contact_uuid TO uuid".to_string(),
            ))
            .await?;

        // Merge legacy customers into the new customers table (keep richer contact data on conflicts).
        manager
            .get_connection()
            .execute(Statement::from_string(
                backend,
                r#"
                INSERT INTO customers (
                    uuid,
                    user_name,
                    company,
                    position,
                    phone_number,
                    email,
                    last_contact,
                    value_level,
                    creator_uuid,
                    status,
                    inserted_at,
                    updated_at
                )
                SELECT
                    c.uuid,
                    c.name,
                    '' AS company,
                    '' AS position,
                    COALESCE(c.phone_number, '') AS phone_number,
                    '' AS email,
                    c.inserted_at AS last_contact,
                    1 AS value_level,
                    c.uuid AS creator_uuid,
                    CASE
                        WHEN c.status = 'active' THEN 1
                        WHEN c.status = 'inactive' THEN 3
                        ELSE 2
                    END AS status,
                    c.inserted_at,
                    c.updated_at
                FROM customers_legacy c
                ON CONFLICT (uuid) DO NOTHING;
                "#
                .to_string(),
            ))
            .await?;

        // Drop the legacy table after merge.
        manager
            .get_connection()
            .execute(Statement::from_string(
                backend,
                "DROP TABLE IF EXISTS customers_legacy CASCADE".to_string(),
            ))
            .await?;

        // Backfill orders.customer_id from contact_uuid (now referencing customers.uuid).
        manager
            .get_connection()
            .execute(Statement::from_string(
                backend,
                "UPDATE orders SET customer_id = contact_uuid WHERE customer_id IS NULL AND contact_uuid IS NOT NULL".to_string(),
            ))
            .await?;

        // Recreate foreign keys to the new customers table.
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_service_requests_contact")
                    .from(ServiceRequests::Table, ServiceRequests::ContactUuid)
                    .to(Customers::Table, Customers::Uuid)
                    .on_delete(ForeignKeyAction::Restrict)
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

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_service_requests_contact")
                    .table(ServiceRequests::Table)
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

        let backend = manager.get_database_backend();

        manager
            .get_connection()
            .execute(Statement::from_string(
                backend,
                "ALTER TABLE IF EXISTS customers RENAME TO contacts".to_string(),
            ))
            .await?;
        manager
            .get_connection()
            .execute(Statement::from_string(
                backend,
                "ALTER TABLE contacts RENAME COLUMN uuid TO contact_uuid".to_string(),
            ))
            .await?;

        // Recreate the legacy customers table schema.
        manager
            .create_table(
                Table::create()
                    .table(Customers::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Customers::Uuid)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Customers::Name).string().not_null())
                    .col(ColumnDef::new(Customers::PhoneNumber).string().null())
                    .col(ColumnDef::new(Customers::Status).string().not_null().default("active"))
                    .col(ColumnDef::new(Customers::Notes).text().null())
                    .col(
                        ColumnDef::new(Customers::InsertedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Customers::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Best-effort backfill legacy customers from contacts.
        manager
            .get_connection()
            .execute(Statement::from_string(
                backend,
                r#"
                INSERT INTO customers (uuid, name, phone_number, status, notes, inserted_at, updated_at)
                SELECT
                    c.contact_uuid,
                    c.user_name,
                    c.phone_number,
                    CASE
                        WHEN c.status = 3 THEN 'inactive'
                        ELSE 'active'
                    END,
                    NULL,
                    c.inserted_at,
                    c.updated_at
                FROM contacts c
                ON CONFLICT (uuid) DO NOTHING;
                "#
                .to_string(),
            ))
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_service_requests_contact")
                    .from(ServiceRequests::Table, ServiceRequests::ContactUuid)
                    .to(Contacts::Table, Contacts::ContactUuid)
                    .on_delete(ForeignKeyAction::Restrict)
                    .to_owned(),
            )
            .await?;
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_orders_contact")
                    .from(Orders::Table, Orders::ContactUuid)
                    .to(Contacts::Table, Contacts::ContactUuid)
                    .on_delete(ForeignKeyAction::Restrict)
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
enum ServiceRequests {
    Table,
    ContactUuid,
}

#[derive(DeriveIden)]
enum Orders {
    Table,
    ContactUuid,
    CustomerId,
}

#[derive(DeriveIden)]
enum Customers {
    Table,
    Uuid,
    Name,
    PhoneNumber,
    Status,
    Notes,
    InsertedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Contacts {
    Table,
    ContactUuid,
}
