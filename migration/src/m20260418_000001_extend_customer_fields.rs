use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::{ConnectionTrait, Statement};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.alter_table(extend_customers_table()).await?;

        let backend = manager.get_database_backend();
        manager
            .get_connection()
            .execute(Statement::from_string(
                backend,
                "UPDATE customers SET tags = '[]'::jsonb WHERE tags IS NULL".to_string(),
            ))
            .await?;
        manager
            .get_connection()
            .execute(Statement::from_string(
                backend,
                "UPDATE customers SET follow_up_status = 'pending' WHERE follow_up_status IS NULL OR follow_up_status = ''".to_string(),
            ))
            .await?;

        manager.create_index(follow_up_status_index()).await?;
        manager.create_index(community_index()).await?;
        manager.create_index(last_service_at_index()).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_customers_last_service_at")
                    .table(Customers::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_index(
                Index::drop()
                    .name("idx_customers_community")
                    .table(Customers::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_index(
                Index::drop()
                    .name("idx_customers_follow_up_status")
                    .table(Customers::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Customers::Table)
                    .drop_column(Customers::FollowUpStatus)
                    .drop_column(Customers::LastServiceAt)
                    .drop_column(Customers::Tags)
                    .drop_column(Customers::ServiceNeed)
                    .drop_column(Customers::HouseAreaSqm)
                    .drop_column(Customers::Building)
                    .drop_column(Customers::Community)
                    .drop_column(Customers::Address)
                    .to_owned(),
            )
            .await
    }
}

fn extend_customers_table() -> TableAlterStatement {
    Table::alter()
        .table(Customers::Table)
        .add_column_if_not_exists(ColumnDef::new(Customers::Address).text().null())
        .add_column_if_not_exists(ColumnDef::new(Customers::Community).string().null())
        .add_column_if_not_exists(ColumnDef::new(Customers::Building).string().null())
        .add_column_if_not_exists(ColumnDef::new(Customers::HouseAreaSqm).integer().null())
        .add_column_if_not_exists(ColumnDef::new(Customers::ServiceNeed).text().null())
        .add_column_if_not_exists(
            ColumnDef::new(Customers::Tags)
                .json_binary()
                .not_null()
                .default(Expr::cust("'[]'::jsonb")),
        )
        .add_column_if_not_exists(
            ColumnDef::new(Customers::LastServiceAt)
                .timestamp_with_time_zone()
                .null(),
        )
        .add_column_if_not_exists(
            ColumnDef::new(Customers::FollowUpStatus)
                .string()
                .not_null()
                .default("pending"),
        )
        .to_owned()
}

fn follow_up_status_index() -> IndexCreateStatement {
    Index::create()
        .name("idx_customers_follow_up_status")
        .table(Customers::Table)
        .col(Customers::FollowUpStatus)
        .if_not_exists()
        .to_owned()
}

fn community_index() -> IndexCreateStatement {
    Index::create()
        .name("idx_customers_community")
        .table(Customers::Table)
        .col(Customers::Community)
        .if_not_exists()
        .to_owned()
}

fn last_service_at_index() -> IndexCreateStatement {
    Index::create()
        .name("idx_customers_last_service_at")
        .table(Customers::Table)
        .col(Customers::LastServiceAt)
        .if_not_exists()
        .to_owned()
}

#[derive(DeriveIden)]
enum Customers {
    Table,
    Address,
    Community,
    Building,
    HouseAreaSqm,
    ServiceNeed,
    Tags,
    LastServiceAt,
    FollowUpStatus,
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm_migration::sea_orm::DbBackend;

    #[test]
    fn alter_table_sql_includes_customer_extension_fields() {
        let sql = DbBackend::Postgres
            .build(&extend_customers_table())
            .to_string();

        assert!(sql.contains(r#"ADD COLUMN IF NOT EXISTS "address" text"#));
        assert!(sql.contains(r#"ADD COLUMN IF NOT EXISTS "community" varchar"#));
        assert!(sql.contains(r#"ADD COLUMN IF NOT EXISTS "building" varchar"#));
        assert!(sql.contains(r#"ADD COLUMN IF NOT EXISTS "house_area_sqm" integer"#));
        assert!(sql.contains(r#"ADD COLUMN IF NOT EXISTS "service_need" text"#));
        assert!(
            sql.contains(r#"ADD COLUMN IF NOT EXISTS "tags" jsonb NOT NULL DEFAULT '[]'::jsonb"#)
        );
        assert!(
            sql.contains(r#"ADD COLUMN IF NOT EXISTS "last_service_at" timestamp with time zone"#)
        );
        assert!(sql.contains(
            r#"ADD COLUMN IF NOT EXISTS "follow_up_status" varchar NOT NULL DEFAULT 'pending'"#
        ));
    }

    #[test]
    fn index_sql_targets_expected_customer_columns() {
        let follow_up_sql = DbBackend::Postgres
            .build(&follow_up_status_index())
            .to_string();
        let community_sql = DbBackend::Postgres.build(&community_index()).to_string();
        let last_service_sql = DbBackend::Postgres
            .build(&last_service_at_index())
            .to_string();

        assert!(follow_up_sql
            .contains(r#"CREATE INDEX IF NOT EXISTS "idx_customers_follow_up_status""#));
        assert!(follow_up_sql.contains(r#"("follow_up_status")"#));
        assert!(community_sql.contains(r#"CREATE INDEX IF NOT EXISTS "idx_customers_community""#));
        assert!(community_sql.contains(r#"("community")"#));
        assert!(last_service_sql
            .contains(r#"CREATE INDEX IF NOT EXISTS "idx_customers_last_service_at""#));
        assert!(last_service_sql.contains(r#"("last_service_at")"#));
    }
}
