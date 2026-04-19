use leptos::prelude::*;
use server_fn::ServerFnError;
use shared::order::{
    CancelOrderRequest, CreateOrderFromRequest, Order, OrderChangeLogDto, OrderQuery,
    UpdateOrderAssignment, UpdateOrderRequest, UpdateOrderSettlement, UpdateOrderStatus,
};
use shared::ListResult;

#[cfg(feature = "ssr")]
mod ssr {
    pub use backend::application::commands::crm::order_service::OrderAppService;
    pub use backend::application::queries::crm::order_service::OrderQueryService;
    pub use backend::infrastructure::db::Database;
    pub use backend::infrastructure::queries::crm::order_query_impl::SeaOrmOrderQuery;
    pub use backend::infrastructure::queries::crm::service_request_query_impl::SeaOrmServiceRequestQuery;
    pub use backend::infrastructure::repositories::crm::order_repository_impl::SeaOrmOrderRepository;
    pub use backend::infrastructure::repositories::crm::schedule_repository_impl::SeaOrmScheduleRepository;
    pub use backend::infrastructure::repositories::crm::service_request_repository_impl::SeaOrmServiceRequestRepository;
    pub use backend::infrastructure::tenant::TenantContext;

    pub async fn wait_for_order_projection(
        pool: &Database,
        schema_name: String,
        order_uuid: String,
    ) -> Result<Option<shared::order::Order>, String> {
        use tokio::time::{sleep, Duration};

        let query = SeaOrmOrderQuery::new(pool.connection.clone(), schema_name);
        let service = OrderQueryService::new(query);

        for _ in 0..20 {
            if let Some(order) = service.fetch_order(order_uuid.clone()).await? {
                return Ok(Some(order));
            }
            sleep(Duration::from_millis(50)).await;
        }

        Ok(None)
    }
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
    name = GetOrderChangeLogsFn,
    prefix = "/api",
    endpoint = "/get_order_change_logs",
)]
pub async fn get_order_change_logs(uuid: String) -> Result<Vec<OrderChangeLogDto>, ServerFnError> {
    use self::ssr::*;
    use axum::Extension;
    use leptos_axum::extract;

    let Extension(tenant): Extension<TenantContext> = extract().await?;
    let pool = expect_context::<Database>();
    let query = SeaOrmOrderQuery::new(pool.connection.clone(), tenant.schema_name);
    let service = OrderQueryService::new(query);

    service
        .fetch_order_change_logs(uuid)
        .await
        .map_err(ServerFnError::new)
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

    let Extension(current_user): Extension<shared::user::User> = extract().await?;
    let Extension(tenant): Extension<TenantContext> = extract().await?;
    let pool = expect_context::<Database>();
    let schema_name = tenant.schema_name.clone();
    let order_repo = SeaOrmOrderRepository::new(pool.connection.clone(), schema_name.clone());
    let schedule_repo = SeaOrmScheduleRepository::new(pool.connection.clone(), schema_name.clone());
    let request_query =
        SeaOrmServiceRequestQuery::new(pool.connection.clone(), schema_name.clone());
    let request_repo =
        SeaOrmServiceRequestRepository::new(pool.connection.clone(), schema_name.clone());
    let service = OrderAppService::new(order_repo, request_query, request_repo, schedule_repo);

    let result = service
        .create_from_request(payload, Some(current_user.uuid))
        .await
        .map_err(|e| ServerFnError::new(e))?;
    let projected = wait_for_order_projection(&pool, schema_name, result.uuid.clone())
        .await
        .map_err(ServerFnError::new)?;
    Ok(projected.unwrap_or(result))
}

#[server(
    name = UpdateOrderFn,
    prefix = "/api",
    endpoint = "/update_order",
)]
pub async fn update_order(
    uuid: String,
    payload: UpdateOrderRequest,
) -> Result<Order, ServerFnError> {
    use self::ssr::*;
    use axum::Extension;
    use leptos_axum::extract;

    let Extension(current_user): Extension<shared::user::User> = extract().await?;
    let Extension(tenant): Extension<TenantContext> = extract().await?;
    let pool = expect_context::<Database>();
    let order_repo =
        SeaOrmOrderRepository::new(pool.connection.clone(), tenant.schema_name.clone());
    let schedule_repo =
        SeaOrmScheduleRepository::new(pool.connection.clone(), tenant.schema_name.clone());
    let request_query =
        SeaOrmServiceRequestQuery::new(pool.connection.clone(), tenant.schema_name.clone());
    let request_repo =
        SeaOrmServiceRequestRepository::new(pool.connection.clone(), tenant.schema_name);
    let service = OrderAppService::new(order_repo, request_query, request_repo, schedule_repo);

    service
        .update_order(uuid, payload, Some(current_user.uuid))
        .await
        .map_err(ServerFnError::new)
}

#[server(
    name = CancelOrderFn,
    prefix = "/api",
    endpoint = "/cancel_order",
)]
pub async fn cancel_order(
    uuid: String,
    payload: CancelOrderRequest,
) -> Result<Order, ServerFnError> {
    use self::ssr::*;
    use axum::Extension;
    use leptos_axum::extract;

    let Extension(current_user): Extension<shared::user::User> = extract().await?;
    let Extension(tenant): Extension<TenantContext> = extract().await?;
    let pool = expect_context::<Database>();
    let order_repo =
        SeaOrmOrderRepository::new(pool.connection.clone(), tenant.schema_name.clone());
    let schedule_repo =
        SeaOrmScheduleRepository::new(pool.connection.clone(), tenant.schema_name.clone());
    let request_query =
        SeaOrmServiceRequestQuery::new(pool.connection.clone(), tenant.schema_name.clone());
    let request_repo =
        SeaOrmServiceRequestRepository::new(pool.connection.clone(), tenant.schema_name);
    let service = OrderAppService::new(order_repo, request_query, request_repo, schedule_repo);

    service
        .cancel_order(uuid, payload, Some(current_user.uuid))
        .await
        .map_err(ServerFnError::new)
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

    let Extension(current_user): Extension<shared::user::User> = extract().await?;
    let Extension(tenant): Extension<TenantContext> = extract().await?;
    let pool = expect_context::<Database>();
    let order_repo =
        SeaOrmOrderRepository::new(pool.connection.clone(), tenant.schema_name.clone());
    let schedule_repo =
        SeaOrmScheduleRepository::new(pool.connection.clone(), tenant.schema_name.clone());
    let request_query =
        SeaOrmServiceRequestQuery::new(pool.connection.clone(), tenant.schema_name.clone());
    let request_repo =
        SeaOrmServiceRequestRepository::new(pool.connection.clone(), tenant.schema_name);
    let service = OrderAppService::new(order_repo, request_query, request_repo, schedule_repo);

    let result = service
        .update_status(uuid, payload, Some(current_user.uuid))
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

    let Extension(current_user): Extension<shared::user::User> = extract().await?;
    let Extension(tenant): Extension<TenantContext> = extract().await?;
    let pool = expect_context::<Database>();
    let order_repo =
        SeaOrmOrderRepository::new(pool.connection.clone(), tenant.schema_name.clone());
    let schedule_repo =
        SeaOrmScheduleRepository::new(pool.connection.clone(), tenant.schema_name.clone());
    let request_query =
        SeaOrmServiceRequestQuery::new(pool.connection.clone(), tenant.schema_name.clone());
    let request_repo =
        SeaOrmServiceRequestRepository::new(pool.connection.clone(), tenant.schema_name);
    let service = OrderAppService::new(order_repo, request_query, request_repo, schedule_repo);

    let result = service
        .update_assignment(uuid, payload, Some(current_user.uuid))
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

    let Extension(current_user): Extension<shared::user::User> = extract().await?;
    let Extension(tenant): Extension<TenantContext> = extract().await?;
    let pool = expect_context::<Database>();
    let order_repo =
        SeaOrmOrderRepository::new(pool.connection.clone(), tenant.schema_name.clone());
    let schedule_repo =
        SeaOrmScheduleRepository::new(pool.connection.clone(), tenant.schema_name.clone());
    let request_query =
        SeaOrmServiceRequestQuery::new(pool.connection.clone(), tenant.schema_name.clone());
    let request_repo =
        SeaOrmServiceRequestRepository::new(pool.connection.clone(), tenant.schema_name);
    let service = OrderAppService::new(order_repo, request_query, request_repo, schedule_repo);

    let result = service
        .update_settlement(uuid, payload, Some(current_user.uuid))
        .await
        .map_err(|e| ServerFnError::new(e))?;
    Ok(result)
}
