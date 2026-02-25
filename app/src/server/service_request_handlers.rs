use leptos::prelude::*;
use server_fn::ServerFnError;
use shared::service_request::{
    CreateServiceRequest, ServiceRequest, ServiceRequestQuery, UpdateServiceRequest,
    UpdateServiceRequestStatus,
};
#[cfg(feature = "ssr")]
use shared::user::User;
use shared::ListResult;

#[cfg(feature = "ssr")]
mod ssr {
    pub use backend::application::commands::service_request_service::ServiceRequestAppService;
    pub use backend::application::queries::service_request_service::ServiceRequestQueryService;
    pub use backend::infrastructure::db::Database;
    pub use backend::infrastructure::queries::service_request_query_impl::SeaOrmServiceRequestQuery;
    pub use backend::infrastructure::repositories::service_request_repository_impl::SeaOrmServiceRequestRepository;
    pub use backend::infrastructure::tenant::TenantContext;
}

#[server(
    name = FetchServiceRequestsFn,
    prefix = "/api",
    endpoint = "/fetch_service_requests",
)]
pub async fn fetch_service_requests(
    params: ServiceRequestQuery,
) -> Result<ListResult<ServiceRequest>, ServerFnError> {
    use self::ssr::*;
    use axum::Extension;
    use leptos_axum::extract;

    let Extension(tenant): Extension<TenantContext> = extract().await?;
    let pool = expect_context::<Database>();
    let query = SeaOrmServiceRequestQuery::new(pool.connection.clone(), tenant.schema_name);
    let service = ServiceRequestQueryService::new(query);

    let result = service
        .fetch_requests(params)
        .await
        .map_err(|e| ServerFnError::new(e))?;
    Ok(result)
}

#[server(
    name = GetServiceRequestFn,
    prefix = "/api",
    endpoint = "/get_service_request",
)]
pub async fn get_service_request(uuid: String) -> Result<Option<ServiceRequest>, ServerFnError> {
    use self::ssr::*;
    use axum::Extension;
    use leptos_axum::extract;

    let Extension(tenant): Extension<TenantContext> = extract().await?;
    let pool = expect_context::<Database>();
    let query = SeaOrmServiceRequestQuery::new(pool.connection.clone(), tenant.schema_name);
    let service = ServiceRequestQueryService::new(query);

    let result = service
        .fetch_request(uuid)
        .await
        .map_err(|e| ServerFnError::new(e))?;
    Ok(result)
}

#[server(
    name = CreateServiceRequestFn,
    prefix = "/api",
    endpoint = "/create_service_request",
)]
pub async fn create_service_request(
    payload: CreateServiceRequest,
) -> Result<ServiceRequest, ServerFnError> {
    use self::ssr::*;
    use axum::Extension;
    use leptos_axum::extract;

    let Extension(current_user): Extension<User> = extract().await?;
    let Extension(tenant): Extension<TenantContext> = extract().await?;
    let pool = expect_context::<Database>();
    let repo = SeaOrmServiceRequestRepository::new(pool.connection.clone(), tenant.schema_name);
    let service = ServiceRequestAppService::new(repo);

    let result = service
        .create_service_request(payload, current_user.uuid)
        .await
        .map_err(|e| ServerFnError::new(e))?;
    Ok(result)
}

#[server(
    name = UpdateServiceRequestFn,
    prefix = "/api",
    endpoint = "/update_service_request",
)]
pub async fn update_service_request(
    payload: UpdateServiceRequest,
) -> Result<ServiceRequest, ServerFnError> {
    use self::ssr::*;
    use axum::Extension;
    use leptos_axum::extract;

    let Extension(tenant): Extension<TenantContext> = extract().await?;
    let pool = expect_context::<Database>();
    let repo = SeaOrmServiceRequestRepository::new(pool.connection.clone(), tenant.schema_name);
    let service = ServiceRequestAppService::new(repo);

    let result = service
        .update_service_request(payload)
        .await
        .map_err(|e| ServerFnError::new(e))?;
    Ok(result)
}

#[server(
    name = UpdateServiceRequestStatusFn,
    prefix = "/api",
    endpoint = "/update_service_request_status",
)]
pub async fn update_service_request_status(
    uuid: String,
    payload: UpdateServiceRequestStatus,
) -> Result<ServiceRequest, ServerFnError> {
    use self::ssr::*;
    use axum::Extension;
    use leptos_axum::extract;

    let Extension(tenant): Extension<TenantContext> = extract().await?;
    let pool = expect_context::<Database>();
    let repo = SeaOrmServiceRequestRepository::new(pool.connection.clone(), tenant.schema_name);
    let service = ServiceRequestAppService::new(repo);

    let result = service
        .update_service_request_status(uuid, payload)
        .await
        .map_err(|e| ServerFnError::new(e))?;
    Ok(result)
}
