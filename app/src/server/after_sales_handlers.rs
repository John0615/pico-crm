use leptos::prelude::*;
use server_fn::ServerFnError;
use shared::after_sales::{
    AfterSalesCase, CreateAfterSalesCaseRequest, UpdateAfterSalesRefundRequest,
};

#[cfg(feature = "ssr")]
mod ssr {
    pub use backend::application::commands::crm::after_sales_service::AfterSalesCaseAppService;
    pub use backend::application::queries::crm::after_sales_service::AfterSalesCaseQueryService;
    pub use backend::domain::crm::order::OrderRepository;
    pub use backend::infrastructure::db::Database;
    pub use backend::infrastructure::queries::crm::after_sales_query_impl::SeaOrmAfterSalesCaseQuery;
    pub use backend::infrastructure::repositories::crm::after_sales_repository_impl::SeaOrmAfterSalesCaseRepository;
    pub use backend::infrastructure::repositories::crm::order_repository_impl::SeaOrmOrderRepository;
    pub use backend::infrastructure::tenant::TenantContext;
}

#[server(
    name = FetchAfterSalesCasesFn,
    prefix = "/api",
    endpoint = "/fetch_after_sales_cases",
)]
pub async fn fetch_after_sales_cases(
    order_uuid: String,
) -> Result<Vec<AfterSalesCase>, ServerFnError> {
    use self::ssr::*;
    use axum::Extension;
    use leptos_axum::extract;
    use shared::user::User;

    let Extension(current_user): Extension<User> = extract().await?;
    if current_user.is_admin.unwrap_or(false)
        || (current_user.role != "operator" && current_user.role != "merchant")
    {
        return Err(ServerFnError::new("无权限查看售后工单".to_string()));
    }

    let Extension(tenant): Extension<TenantContext> = extract().await?;
    let pool = expect_context::<Database>();
    let query = SeaOrmAfterSalesCaseQuery::new(pool.connection.clone(), tenant.schema_name);
    let service = AfterSalesCaseQueryService::new(query);
    service
        .fetch_cases(order_uuid)
        .await
        .map_err(ServerFnError::new)
}

#[server(
    name = CreateAfterSalesCaseFn,
    prefix = "/api",
    endpoint = "/create_after_sales_case",
)]
pub async fn create_after_sales_case(
    order_uuid: String,
    payload: CreateAfterSalesCaseRequest,
) -> Result<AfterSalesCase, ServerFnError> {
    use self::ssr::*;
    use axum::Extension;
    use leptos_axum::extract;
    use shared::user::User;

    let Extension(current_user): Extension<User> = extract().await?;
    if current_user.is_admin.unwrap_or(false)
        || (current_user.role != "operator" && current_user.role != "merchant")
    {
        return Err(ServerFnError::new("无权限创建售后工单".to_string()));
    }

    let Extension(tenant): Extension<TenantContext> = extract().await?;
    let pool = expect_context::<Database>();
    let order_repo =
        SeaOrmOrderRepository::new(pool.connection.clone(), tenant.schema_name.clone());
    let order = order_repo
        .find_order(order_uuid.clone())
        .await
        .map_err(ServerFnError::new)?;
    if order.is_none() {
        return Err(ServerFnError::new("订单不存在".to_string()));
    }

    let repo = SeaOrmAfterSalesCaseRepository::new(pool.connection.clone(), tenant.schema_name);
    let service = AfterSalesCaseAppService::new(repo);
    service
        .create_case(order_uuid, payload, Some(current_user.uuid))
        .await
        .map_err(ServerFnError::new)
}

#[server(
    name = UpdateAfterSalesRefundFn,
    prefix = "/api",
    endpoint = "/update_after_sales_refund",
)]
pub async fn update_after_sales_refund(
    case_uuid: String,
    payload: UpdateAfterSalesRefundRequest,
) -> Result<AfterSalesCase, ServerFnError> {
    use self::ssr::*;
    use axum::Extension;
    use leptos_axum::extract;
    use shared::user::User;

    let Extension(current_user): Extension<User> = extract().await?;
    if current_user.is_admin.unwrap_or(false)
        || (current_user.role != "operator" && current_user.role != "merchant")
    {
        return Err(ServerFnError::new("无权限记录退款信息".to_string()));
    }

    let Extension(tenant): Extension<TenantContext> = extract().await?;
    let pool = expect_context::<Database>();
    let repo = SeaOrmAfterSalesCaseRepository::new(pool.connection.clone(), tenant.schema_name);
    let service = AfterSalesCaseAppService::new(repo);
    service
        .update_refund(case_uuid, payload)
        .await
        .map_err(ServerFnError::new)
}
