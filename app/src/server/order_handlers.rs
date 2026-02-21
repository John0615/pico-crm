use leptos::prelude::*;
use server_fn::ServerFnError;
use shared::order::{
    CreateOrderFromRequest, Order, OrderQuery, UpdateOrderAssignment, UpdateOrderSettlement,
    UpdateOrderStatus,
};
use shared::ListResult;

#[cfg(feature = "ssr")]
mod ssr {
    pub use backend::application::commands::order_service::OrderAppService;
    pub use backend::application::queries::order_service::OrderQueryService;
    pub use backend::infrastructure::db::Database;
    pub use backend::infrastructure::queries::order_query_impl::SeaOrmOrderQuery;
    pub use backend::infrastructure::queries::service_request_query_impl::SeaOrmServiceRequestQuery;
    pub use backend::infrastructure::repositories::order_repository_impl::SeaOrmOrderRepository;
    pub use backend::infrastructure::repositories::service_request_repository_impl::SeaOrmServiceRequestRepository;
    pub use backend::infrastructure::tenant::TenantContext;
}

#[server(
    name = FetchOrdersFn,
    prefix = "/api",
    endpoint = "/fetch_orders",
)]
pub async fn fetch_orders(params: OrderQuery) -> Result<ListResult<Order>, ServerFnError> {
    use self::ssr::*;
    use axum::Extension;
    use leptos_axum::extract;

    let Extension(tenant): Extension<TenantContext> = extract().await?;
    let pool = expect_context::<Database>();
    let query = SeaOrmOrderQuery::new(pool.connection.clone(), tenant.schema_name);
    let service = OrderQueryService::new(query);

    let result = service
        .fetch_orders(params)
        .await
        .map_err(|e| ServerFnError::new(e))?;
    Ok(result)
}

#[server(
    name = GetOrderFn,
    prefix = "/api",
    endpoint = "/get_order",
)]
pub async fn get_order(uuid: String) -> Result<Option<Order>, ServerFnError> {
    use self::ssr::*;
    use axum::Extension;
    use leptos_axum::extract;

    let Extension(tenant): Extension<TenantContext> = extract().await?;
    let pool = expect_context::<Database>();
    let query = SeaOrmOrderQuery::new(pool.connection.clone(), tenant.schema_name);
    let service = OrderQueryService::new(query);

    let result = service
        .fetch_order(uuid)
        .await
        .map_err(|e| ServerFnError::new(e))?;
    Ok(result)
}

#[server(
    name = CreateOrderFromRequestFn,
    prefix = "/api",
    endpoint = "/create_order_from_request",
)]
pub async fn create_order_from_request(
    payload: CreateOrderFromRequest,
) -> Result<Order, ServerFnError> {
    use self::ssr::*;
    use axum::Extension;
    use leptos_axum::extract;

    let Extension(tenant): Extension<TenantContext> = extract().await?;
    let pool = expect_context::<Database>();
    let order_repo = SeaOrmOrderRepository::new(pool.connection.clone(), tenant.schema_name.clone());
    let request_query =
        SeaOrmServiceRequestQuery::new(pool.connection.clone(), tenant.schema_name.clone());
    let request_repo =
        SeaOrmServiceRequestRepository::new(pool.connection.clone(), tenant.schema_name);
    let service = OrderAppService::new(order_repo, request_query, request_repo);

    let result = service
        .create_from_request(payload)
        .await
        .map_err(|e| ServerFnError::new(e))?;
    Ok(result)
}

#[server(
    name = UpdateOrderStatusFn,
    prefix = "/api",
    endpoint = "/update_order_status",
)]
pub async fn update_order_status(
    uuid: String,
    payload: UpdateOrderStatus,
) -> Result<Order, ServerFnError> {
    use self::ssr::*;
    use axum::Extension;
    use leptos_axum::extract;

    let Extension(tenant): Extension<TenantContext> = extract().await?;
    let pool = expect_context::<Database>();
    let order_repo = SeaOrmOrderRepository::new(pool.connection.clone(), tenant.schema_name.clone());
    let request_query =
        SeaOrmServiceRequestQuery::new(pool.connection.clone(), tenant.schema_name.clone());
    let request_repo =
        SeaOrmServiceRequestRepository::new(pool.connection.clone(), tenant.schema_name);
    let service = OrderAppService::new(order_repo, request_query, request_repo);

    let result = service
        .update_status(uuid, payload)
        .await
        .map_err(|e| ServerFnError::new(e))?;
    Ok(result)
}

#[server(
    name = UpdateOrderAssignmentFn,
    prefix = "/api",
    endpoint = "/update_order_assignment",
)]
pub async fn update_order_assignment(
    uuid: String,
    payload: UpdateOrderAssignment,
) -> Result<Order, ServerFnError> {
    use self::ssr::*;
    use axum::Extension;
    use leptos_axum::extract;

    let Extension(tenant): Extension<TenantContext> = extract().await?;
    let pool = expect_context::<Database>();
    let order_repo = SeaOrmOrderRepository::new(pool.connection.clone(), tenant.schema_name.clone());
    let request_query =
        SeaOrmServiceRequestQuery::new(pool.connection.clone(), tenant.schema_name.clone());
    let request_repo =
        SeaOrmServiceRequestRepository::new(pool.connection.clone(), tenant.schema_name);
    let service = OrderAppService::new(order_repo, request_query, request_repo);

    let result = service
        .update_assignment(uuid, payload)
        .await
        .map_err(|e| ServerFnError::new(e))?;
    Ok(result)
}

#[server(
    name = UpdateOrderSettlementFn,
    prefix = "/api",
    endpoint = "/update_order_settlement",
)]
pub async fn update_order_settlement(
    uuid: String,
    payload: UpdateOrderSettlement,
) -> Result<Order, ServerFnError> {
    use self::ssr::*;
    use axum::Extension;
    use leptos_axum::extract;

    let Extension(tenant): Extension<TenantContext> = extract().await?;
    let pool = expect_context::<Database>();
    let order_repo = SeaOrmOrderRepository::new(pool.connection.clone(), tenant.schema_name.clone());
    let request_query =
        SeaOrmServiceRequestQuery::new(pool.connection.clone(), tenant.schema_name.clone());
    let request_repo =
        SeaOrmServiceRequestRepository::new(pool.connection.clone(), tenant.schema_name);
    let service = OrderAppService::new(order_repo, request_query, request_repo);

    let result = service
        .update_settlement(uuid, payload)
        .await
        .map_err(|e| ServerFnError::new(e))?;
    Ok(result)
}
