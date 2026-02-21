use leptos::prelude::*;
use server_fn::ServerFnError;
use shared::admin::{TenantMigrationRequest, TenantMigrationResponse};

#[server(
    name = RunTenantMigrations,
    prefix = "/api/admin",
    endpoint = "/tenants/migrations/run",
)]
pub async fn run_tenant_migrations(
    request: Option<TenantMigrationRequest>,
) -> Result<TenantMigrationResponse, ServerFnError> {
    use backend::application::commands::admin_tenant_migration_service::AdminTenantMigrationService;
    use backend::infrastructure::db::Database;

    let pool = expect_context::<Database>();
    let service = AdminTenantMigrationService::new(pool.get_connection().clone());
    let request = request.unwrap_or_default();

    service
        .run(request)
        .await
        .map_err(|e| ServerFnError::new(format!("执行租户迁移失败: {}", e)))
}
