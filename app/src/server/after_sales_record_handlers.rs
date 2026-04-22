use leptos::prelude::*;
use server_fn::ServerFnError;
use shared::after_sales::{AfterSalesCaseRecord, CreateAfterSalesCaseRecordRequest};

#[cfg(feature = "ssr")]
mod ssr {
    pub use backend::application::commands::crm::after_sales_record_service::AfterSalesCaseRecordAppService;
    pub use backend::application::queries::crm::after_sales_record_service::AfterSalesCaseRecordQueryService;
    pub use backend::infrastructure::db::Database;
    pub use backend::infrastructure::queries::crm::after_sales_record_query_impl::SeaOrmAfterSalesCaseRecordQuery;
    pub use backend::infrastructure::repositories::crm::after_sales_record_repository_impl::SeaOrmAfterSalesCaseRecordRepository;
    pub use backend::infrastructure::tenant::TenantContext;
}

#[server(
    name = FetchAfterSalesCaseRecordsFn,
    prefix = "/api",
    endpoint = "/fetch_after_sales_case_records",
)]
pub async fn fetch_after_sales_case_records(
    case_uuid: String,
) -> Result<Vec<AfterSalesCaseRecord>, ServerFnError> {
    use self::ssr::*;
    use axum::Extension;
    use leptos_axum::extract;
    use shared::user::User;

    let Extension(current_user): Extension<User> = extract().await?;
    if current_user.is_admin.unwrap_or(false)
        || (current_user.role != "operator" && current_user.role != "merchant")
    {
        return Err(ServerFnError::new("无权限查看售后处理记录".to_string()));
    }

    let Extension(tenant): Extension<TenantContext> = extract().await?;
    let pool = expect_context::<Database>();
    let query = SeaOrmAfterSalesCaseRecordQuery::new(pool.connection.clone(), tenant.schema_name);
    let service = AfterSalesCaseRecordQueryService::new(query);
    service
        .fetch_records(case_uuid)
        .await
        .map_err(ServerFnError::new)
}

#[server(
    name = CreateAfterSalesCaseRecordFn,
    prefix = "/api",
    endpoint = "/create_after_sales_case_record",
)]
pub async fn create_after_sales_case_record(
    case_uuid: String,
    payload: CreateAfterSalesCaseRecordRequest,
) -> Result<AfterSalesCaseRecord, ServerFnError> {
    use self::ssr::*;
    use axum::Extension;
    use leptos_axum::extract;
    use shared::user::User;

    let Extension(current_user): Extension<User> = extract().await?;
    if current_user.is_admin.unwrap_or(false)
        || (current_user.role != "operator" && current_user.role != "merchant")
    {
        return Err(ServerFnError::new("无权限记录售后处理过程".to_string()));
    }

    let Extension(tenant): Extension<TenantContext> = extract().await?;
    let pool = expect_context::<Database>();
    let repo =
        SeaOrmAfterSalesCaseRecordRepository::new(pool.connection.clone(), tenant.schema_name);
    let service = AfterSalesCaseRecordAppService::new(repo);
    service
        .create_record(case_uuid, payload, Some(current_user.uuid))
        .await
        .map_err(ServerFnError::new)
}
