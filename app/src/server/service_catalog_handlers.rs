use leptos::prelude::*;
use server_fn::ServerFnError;
use shared::{
    service_catalog::{
        CreateServiceCatalogRequest, ServiceCatalog, ServiceCatalogQuery,
        UpdateServiceCatalogRequest,
    },
    ListResult,
};

#[cfg(feature = "ssr")]
mod ssr {
    pub use backend::application::commands::crm::service_catalog_service::ServiceCatalogAppService;
    pub use backend::application::queries::crm::service_catalog_service::ServiceCatalogQueryService;
    pub use backend::infrastructure::db::Database;
    pub use backend::infrastructure::queries::crm::service_catalog_query_impl::SeaOrmServiceCatalogQuery;
    pub use backend::infrastructure::repositories::crm::service_catalog_repository_impl::SeaOrmServiceCatalogRepository;
    pub use backend::infrastructure::tenant::TenantContext;
}

#[server(
    name = FetchServiceCatalogsFn,
    prefix = "/api",
    endpoint = "/fetch_service_catalogs",
)]
pub async fn fetch_service_catalogs(
    page: u64,
    page_size: u64,
    active_only: Option<bool>,
) -> Result<ListResult<ServiceCatalog>, ServerFnError> {
    use self::ssr::*;
    use axum::Extension;
    use leptos_axum::extract;

    let Extension(tenant): Extension<TenantContext> = extract().await?;
    let pool = expect_context::<Database>();
    let query = SeaOrmServiceCatalogQuery::new(pool.connection.clone(), tenant.merchant_id.clone());
    let service = ServiceCatalogQueryService::new(query);
    let params = ServiceCatalogQuery {
        page,
        page_size,
        active_only,
    };
    let (items, total) = service
        .fetch_service_catalogs(params)
        .await
        .map_err(ServerFnError::new)?;
    Ok(ListResult { items, total })
}

#[server(
    name = CreateServiceCatalogFn,
    prefix = "/api",
    endpoint = "/create_service_catalog",
)]
pub async fn create_service_catalog(
    payload: CreateServiceCatalogRequest,
) -> Result<ServiceCatalog, ServerFnError> {
    use self::ssr::*;
    use axum::Extension;
    use leptos_axum::extract;
    use shared::user::User;

    let Extension(current_user): Extension<User> = extract().await?;
    if current_user.is_admin.unwrap_or(false)
        || (current_user.role != "operator" && current_user.role != "merchant")
    {
        return Err(ServerFnError::new("无权限维护服务项目".to_string()));
    }

    let Extension(tenant): Extension<TenantContext> = extract().await?;
    let pool = expect_context::<Database>();
    let repo =
        SeaOrmServiceCatalogRepository::new(pool.connection.clone(), tenant.merchant_id.clone());
    let service = ServiceCatalogAppService::new(repo);
    service
        .create_service_catalog(payload)
        .await
        .map_err(ServerFnError::new)
}

#[server(
    name = UpdateServiceCatalogFn,
    prefix = "/api",
    endpoint = "/update_service_catalog",
)]
pub async fn update_service_catalog(
    uuid: String,
    payload: UpdateServiceCatalogRequest,
) -> Result<ServiceCatalog, ServerFnError> {
    use self::ssr::*;
    use axum::Extension;
    use leptos_axum::extract;
    use shared::user::User;

    let Extension(current_user): Extension<User> = extract().await?;
    if current_user.is_admin.unwrap_or(false)
        || (current_user.role != "operator" && current_user.role != "merchant")
    {
        return Err(ServerFnError::new("无权限维护服务项目".to_string()));
    }

    let Extension(tenant): Extension<TenantContext> = extract().await?;
    let pool = expect_context::<Database>();
    let repo =
        SeaOrmServiceCatalogRepository::new(pool.connection.clone(), tenant.merchant_id.clone());
    let service = ServiceCatalogAppService::new(repo);
    service
        .update_service_catalog(uuid, payload)
        .await
        .map_err(ServerFnError::new)
}

#[server(
    name = DeleteServiceCatalogFn,
    prefix = "/api",
    endpoint = "/delete_service_catalog",
)]
pub async fn delete_service_catalog(uuid: String) -> Result<(), ServerFnError> {
    use self::ssr::*;
    use axum::Extension;
    use leptos_axum::extract;
    use shared::user::User;

    let Extension(current_user): Extension<User> = extract().await?;
    if current_user.is_admin.unwrap_or(false)
        || (current_user.role != "operator" && current_user.role != "merchant")
    {
        return Err(ServerFnError::new("无权限维护服务项目".to_string()));
    }

    let Extension(tenant): Extension<TenantContext> = extract().await?;
    let pool = expect_context::<Database>();
    let repo =
        SeaOrmServiceCatalogRepository::new(pool.connection.clone(), tenant.merchant_id.clone());
    let service = ServiceCatalogAppService::new(repo);
    service
        .delete_service_catalog(uuid)
        .await
        .map_err(ServerFnError::new)
}
