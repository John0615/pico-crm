use crate::infrastructure::entity::merchant::{Column, Entity};
use crate::infrastructure::tenant::is_safe_schema_name;
use migration::run_tenant_migrations;
use sea_orm::{
    ColumnTrait, ConnectionTrait, DatabaseBackend, DatabaseConnection, EntityTrait, QueryFilter,
    QuerySelect, Statement, TransactionTrait,
};
use sea_orm::entity::prelude::Uuid;
use shared::admin::{TenantMigrationFailure, TenantMigrationRequest, TenantMigrationResponse};

pub struct AdminTenantMigrationService {
    db: DatabaseConnection,
}

impl AdminTenantMigrationService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn run(
        &self,
        request: TenantMigrationRequest,
    ) -> Result<TenantMigrationResponse, String> {
        let schema_names = self.fetch_schema_names(request).await?;
        let total = schema_names.len() as u64;
        let mut migrated = 0_u64;
        let mut failures = Vec::new();

        for schema_name in schema_names {
            if !is_safe_schema_name(&schema_name) {
                failures.push(TenantMigrationFailure {
                    schema_name,
                    error: "invalid schema name".to_string(),
                });
                continue;
            }

            let result = self
                .ensure_schema(&schema_name)
                .await
                .and_then(|_| Ok(()));

            let result = match result {
                Ok(()) => run_tenant_migrations(&self.db, &schema_name)
                    .await
                    .map_err(|e| e.to_string()),
                Err(err) => Err(err),
            };

            match result {
                Ok(()) => migrated += 1,
                Err(err) => failures.push(TenantMigrationFailure {
                    schema_name,
                    error: err,
                }),
            }
        }

        Ok(TenantMigrationResponse {
            total,
            migrated,
            failed: failures.len() as u64,
            failures,
        })
    }

    async fn fetch_schema_names(
        &self,
        request: TenantMigrationRequest,
    ) -> Result<Vec<String>, String> {
        let txn = self
            .db
            .begin()
            .await
            .map_err(|e| format!("begin transaction error: {}", e))?;
        let stmt = Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "SELECT set_config('search_path', $1, true)",
            vec!["public".to_string().into()],
        );
        txn.execute(stmt)
            .await
            .map_err(|e| format!("set search_path error: {}", e))?;

        let mut select = Entity::find().select_only().column(Column::SchemaName);
        if let Some(status) = normalize_optional_string(request.status) {
            select = select.filter(Column::Status.eq(status));
        }
        if let Some(merchant_uuid) = normalize_optional_string(request.merchant_uuid) {
            let parsed = Uuid::parse_str(&merchant_uuid)
                .map_err(|e| format!("invalid merchant uuid: {}", e))?;
            select = select.filter(Column::Uuid.eq(parsed));
        }

        let schema_names = select
            .into_tuple::<String>()
            .all(&txn)
            .await
            .map_err(|e| format!("query merchant schemas error: {}", e))?;

        txn.commit()
            .await
            .map_err(|e| format!("commit transaction error: {}", e))?;

        Ok(schema_names)
    }

    async fn ensure_schema(&self, schema_name: &str) -> Result<(), String> {
        if !is_safe_schema_name(schema_name) {
            return Err("invalid schema name".to_string());
        }
        let stmt = Statement::from_string(
            DatabaseBackend::Postgres,
            format!("CREATE SCHEMA IF NOT EXISTS \"{}\"", schema_name),
        );
        self.db
            .execute(stmt)
            .await
            .map_err(|e| format!("create schema error: {}", e))?;
        Ok(())
    }
}

fn normalize_optional_string(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() || trimmed == "all" {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}
