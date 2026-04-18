use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::{ConnectionTrait, Statement};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column_if_not_exists(
                        ColumnDef::new(Users::EmploymentStatus)
                            .string()
                            .not_null()
                            .default("active"),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(Users::Skills)
                            .json_binary()
                            .not_null()
                            .default(Expr::cust("'[]'::jsonb")),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(Users::ServiceAreas)
                            .json_binary()
                            .not_null()
                            .default(Expr::cust("'[]'::jsonb")),
                    )
                    .add_column_if_not_exists(ColumnDef::new(Users::EmployeeNote).text().null())
                    .add_column_if_not_exists(
                        ColumnDef::new(Users::JoinedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        let backend = manager.get_database_backend();
        manager
            .get_connection()
            .execute(Statement::from_string(
                backend,
                "UPDATE users SET employment_status = 'active' WHERE employment_status IS NULL OR employment_status = ''".to_string(),
            ))
            .await?;
        manager
            .get_connection()
            .execute(Statement::from_string(
                backend,
                "UPDATE users SET skills = '[]'::jsonb WHERE skills IS NULL".to_string(),
            ))
            .await?;
        manager
            .get_connection()
            .execute(Statement::from_string(
                backend,
                "UPDATE users SET service_areas = '[]'::jsonb WHERE service_areas IS NULL"
                    .to_string(),
            ))
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_users_employment_status")
                    .table(Users::Table)
                    .col(Users::EmploymentStatus)
                    .if_not_exists()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_users_employment_status")
                    .table(Users::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .drop_column(Users::JoinedAt)
                    .drop_column(Users::EmployeeNote)
                    .drop_column(Users::ServiceAreas)
                    .drop_column(Users::Skills)
                    .drop_column(Users::EmploymentStatus)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    EmploymentStatus,
    Skills,
    ServiceAreas,
    EmployeeNote,
    JoinedAt,
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm_migration::sea_orm::DbBackend;

    #[test]
    fn alter_table_sql_includes_employee_extension_fields() {
        let sql = DbBackend::Postgres
            .build(
                &Table::alter()
                    .table(Users::Table)
                    .add_column_if_not_exists(
                        ColumnDef::new(Users::EmploymentStatus)
                            .string()
                            .not_null()
                            .default("active"),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(Users::Skills)
                            .json_binary()
                            .not_null()
                            .default(Expr::cust("'[]'::jsonb")),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(Users::ServiceAreas)
                            .json_binary()
                            .not_null()
                            .default(Expr::cust("'[]'::jsonb")),
                    )
                    .add_column_if_not_exists(ColumnDef::new(Users::EmployeeNote).text().null())
                    .add_column_if_not_exists(
                        ColumnDef::new(Users::JoinedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .to_owned(),
            )
            .to_string();

        assert!(sql.contains(
            r#"ADD COLUMN IF NOT EXISTS "employment_status" varchar NOT NULL DEFAULT 'active'"#
        ));
        assert!(
            sql.contains(r#"ADD COLUMN IF NOT EXISTS "skills" jsonb NOT NULL DEFAULT '[]'::jsonb"#)
        );
        assert!(sql.contains(
            r#"ADD COLUMN IF NOT EXISTS "service_areas" jsonb NOT NULL DEFAULT '[]'::jsonb"#
        ));
        assert!(sql.contains(r#"ADD COLUMN IF NOT EXISTS "employee_note" text"#));
        assert!(sql.contains(r#"ADD COLUMN IF NOT EXISTS "joined_at" timestamp with time zone"#));
    }
}
