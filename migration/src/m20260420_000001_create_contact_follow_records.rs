use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(create_table_stmt()).await?;
        manager.create_index(contact_uuid_index()).await?;
        manager.create_index(next_follow_up_at_index()).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_contact_follow_records_next_follow_up_at")
                    .table(ContactFollowRecords::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_index(
                Index::drop()
                    .name("idx_contact_follow_records_contact_uuid")
                    .table(ContactFollowRecords::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(Table::drop().table(ContactFollowRecords::Table).to_owned())
            .await
    }
}

fn create_table_stmt() -> TableCreateStatement {
    Table::create()
        .table(ContactFollowRecords::Table)
        .if_not_exists()
        .col(
            ColumnDef::new(ContactFollowRecords::Uuid)
                .uuid()
                .not_null()
                .primary_key()
                .default(Expr::cust("gen_random_uuid()")),
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
        .col(ColumnDef::new(ContactFollowRecords::Content).text().not_null())
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
                .name("fk_contact_follow_records_contact")
                .from(ContactFollowRecords::Table, ContactFollowRecords::ContactUuid)
                .to(Customers::Table, Customers::Uuid)
                .on_delete(ForeignKeyAction::Cascade),
        )
        .foreign_key(
            ForeignKey::create()
                .name("fk_contact_follow_records_operator")
                .from(ContactFollowRecords::Table, ContactFollowRecords::OperatorUuid)
                .to(Users::Table, Users::Uuid)
                .on_delete(ForeignKeyAction::SetNull),
        )
        .to_owned()
}

fn contact_uuid_index() -> IndexCreateStatement {
    Index::create()
        .name("idx_contact_follow_records_contact_uuid")
        .table(ContactFollowRecords::Table)
        .col(ContactFollowRecords::ContactUuid)
        .if_not_exists()
        .to_owned()
}

fn next_follow_up_at_index() -> IndexCreateStatement {
    Index::create()
        .name("idx_contact_follow_records_next_follow_up_at")
        .table(ContactFollowRecords::Table)
        .col(ContactFollowRecords::NextFollowUpAt)
        .if_not_exists()
        .to_owned()
}

#[derive(DeriveIden)]
enum ContactFollowRecords {
    Table,
    Uuid,
    ContactUuid,
    OperatorUuid,
    Content,
    NextFollowUpAt,
    CreatedAt,
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

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm_migration::sea_orm::DbBackend;

    #[test]
    fn create_table_sql_includes_contact_follow_record_fields() {
        let sql = DbBackend::Postgres.build(&create_table_stmt()).to_string();

        assert!(sql.contains(r#"CREATE TABLE IF NOT EXISTS "contact_follow_records""#));
        assert!(sql.contains(r#""content" text NOT NULL"#));
        assert!(sql.contains(r#""next_follow_up_at" timestamp with time zone"#));
        assert!(sql.contains(r#"CONSTRAINT "fk_contact_follow_records_contact""#));
        assert!(sql.contains(r#"CONSTRAINT "fk_contact_follow_records_operator""#));
        assert!(sql.contains(r#"REFERENCES "customers" ("uuid")"#));
    }

    #[test]
    fn index_sql_targets_contact_follow_record_lookup_columns() {
        let contact_sql = DbBackend::Postgres.build(&contact_uuid_index()).to_string();
        let follow_up_sql = DbBackend::Postgres
            .build(&next_follow_up_at_index())
            .to_string();

        assert!(contact_sql.contains(
            r#"CREATE INDEX IF NOT EXISTS "idx_contact_follow_records_contact_uuid""#
        ));
        assert!(contact_sql.contains(r#"("contact_uuid")"#));
        assert!(follow_up_sql.contains(
            r#"CREATE INDEX IF NOT EXISTS "idx_contact_follow_records_next_follow_up_at""#
        ));
        assert!(follow_up_sql.contains(r#"("next_follow_up_at")"#));
    }
}
