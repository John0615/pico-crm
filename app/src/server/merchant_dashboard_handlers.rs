use leptos::prelude::*;
use server_fn::ServerFnError;
use shared::merchant_dashboard::{MerchantDashboardQuery, MerchantDashboardResponse};

#[cfg(feature = "ssr")]
mod ssr {
    pub use backend::application::queries::platform::merchant_dashboard_service::MerchantDashboardQueryService;
    pub use backend::infrastructure::db::Database;
    pub use backend::infrastructure::tenant::TenantContext;
}

#[server(
    name = FetchMerchantDashboardFn,
    prefix = "/api",
    endpoint = "/merchant_dashboard",
)]
pub async fn fetch_merchant_dashboard(
    query: MerchantDashboardQuery,
) -> Result<MerchantDashboardResponse, ServerFnError> {
    use self::ssr::*;
    use axum::Extension;
    use leptos_axum::extract;

    let Extension(tenant): Extension<TenantContext> = extract().await?;
    let pool = expect_context::<Database>();
    let service = MerchantDashboardQueryService::new(pool.connection.clone(), tenant.schema_name);

    let result = service
        .fetch_dashboard(query)
        .await
        .map_err(|e| ServerFnError::new(e))?;
    Ok(result)
}
