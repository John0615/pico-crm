use leptos::prelude::*;
use server_fn::ServerFnError;
use shared::system_config::{
    SystemConfigCategoryDto, SystemConfigUpdateRequest, SystemConfigUpdateResponse,
};

#[server(
    name = FetchSystemConfig,
    prefix = "/api/admin",
    endpoint = "/system-config",
)]
pub async fn fetch_system_config() -> Result<Vec<SystemConfigCategoryDto>, ServerFnError> {
    use backend::application::queries::system_config_service::SystemConfigQueryService;
    use backend::infrastructure::db::Database;
    use backend::infrastructure::repositories::system_config_repository_impl::SeaOrmSystemConfigRepository;

    let pool = expect_context::<Database>();
    let repo = SeaOrmSystemConfigRepository::new(pool.get_connection().clone());
    let service = SystemConfigQueryService::new(repo);

    let configs = service
        .list_configs()
        .await
        .map_err(|e| ServerFnError::new(format!("查询系统配置失败: {}", e)))?;

    Ok(configs)
}

#[server(
    name = UpdateSystemConfig,
    prefix = "/api/admin",
    endpoint = "/system-config/update",
)]
pub async fn update_system_config(
    request: SystemConfigUpdateRequest,
) -> Result<SystemConfigUpdateResponse, ServerFnError> {
    use backend::application::commands::system_config_service::SystemConfigCommandService;
    use backend::application::mappers::system_config_mapper::to_item_dto;
    use backend::domain::models::system_config::SystemConfigItemUpdateRequest;
    use backend::infrastructure::db::Database;
    use backend::infrastructure::repositories::audit_log_repository_impl::SeaOrmAuditLogRepository;
    use backend::infrastructure::repositories::system_config_repository_impl::SeaOrmSystemConfigRepository;

    let pool = expect_context::<Database>();
    let repo = SeaOrmSystemConfigRepository::new(pool.get_connection().clone());
    let audit_repo = SeaOrmAuditLogRepository::new(pool.get_connection().clone());
    let service = SystemConfigCommandService::new(repo, audit_repo);

    let updates = request
        .items
        .into_iter()
        .map(|item| SystemConfigItemUpdateRequest {
            key: item.key,
            value: item.value,
            reset_to_default: item.reset_to_default.unwrap_or(false),
        })
        .collect();

    let updated_items = service
        .update_items(updates, None, Some("admin".to_string()))
        .await
        .map_err(|e| ServerFnError::new(format!("更新系统配置失败: {}", e)))?;

    let items = updated_items.into_iter().map(to_item_dto).collect();

    Ok(SystemConfigUpdateResponse { items })
}
