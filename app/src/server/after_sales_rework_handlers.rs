use leptos::prelude::*;
use server_fn::ServerFnError;
use shared::after_sales::{AfterSalesRework, CreateAfterSalesReworkRequest};

#[cfg(feature = "ssr")]
mod ssr {
    pub use backend::application::commands::crm::after_sales_rework_service::AfterSalesReworkAppService;
    pub use backend::application::queries::crm::after_sales_rework_service::AfterSalesReworkQueryService;
    pub use backend::infrastructure::db::Database;
    pub use backend::infrastructure::queries::crm::after_sales_rework_query_impl::SeaOrmAfterSalesReworkQuery;
    pub use backend::infrastructure::repositories::crm::after_sales_rework_repository_impl::SeaOrmAfterSalesReworkRepository;
    pub use backend::infrastructure::tenant::TenantContext;
}

#[server(
    name = FetchAfterSalesReworksFn,
    prefix = "/api",
    endpoint = "/fetch_after_sales_reworks",
)]
pub async fn fetch_after_sales_reworks(
    case_uuid: String,
) -> Result<Vec<AfterSalesRework>, ServerFnError> {
    use self::ssr::*;
    use axum::Extension;
    use leptos_axum::extract;
    use shared::user::User;

    let Extension(current_user): Extension<User> = extract().await?;
    if current_user.is_admin.unwrap_or(false)
        || (current_user.role != "operator" && current_user.role != "merchant")
    {
        return Err(ServerFnError::new("无权限查看返工安排".to_string()));
    }

    let Extension(tenant): Extension<TenantContext> = extract().await?;
    let pool = expect_context::<Database>();
    let query =
        SeaOrmAfterSalesReworkQuery::new(pool.connection.clone(), tenant.merchant_id.clone());
    let service = AfterSalesReworkQueryService::new(query);
    service
        .fetch_reworks(case_uuid)
        .await
        .map_err(ServerFnError::new)
}

#[server(
    name = CreateAfterSalesReworkFn,
    prefix = "/api",
    endpoint = "/create_after_sales_rework",
)]
pub async fn create_after_sales_rework(
    case_uuid: String,
    payload: CreateAfterSalesReworkRequest,
) -> Result<AfterSalesRework, ServerFnError> {
    use self::ssr::*;
    use axum::Extension;
    use leptos_axum::extract;
    use shared::user::User;

    let Extension(current_user): Extension<User> = extract().await?;
    if current_user.is_admin.unwrap_or(false)
        || (current_user.role != "operator" && current_user.role != "merchant")
    {
        return Err(ServerFnError::new("无权限记录返工安排".to_string()));
    }

    let Extension(tenant): Extension<TenantContext> = extract().await?;
    let pool = expect_context::<Database>();
    let repo =
        SeaOrmAfterSalesReworkRepository::new(pool.connection.clone(), tenant.merchant_id.clone());
    let service = AfterSalesReworkAppService::new(repo);
    service
        .create_rework(case_uuid, payload)
        .await
        .map_err(ServerFnError::new)
}
