use leptos::prelude::*;
use server_fn::ServerFnError;
use shared::analytics::{
    AnalyticsBreakdownResponse, AnalyticsOverviewResponse, AnalyticsQuery, AnalyticsTrendResponse,
};

#[server(
    name = FetchAdminAnalyticsOverview,
    prefix = "/api/admin/analytics",
    endpoint = "/overview",
)]
pub async fn fetch_admin_analytics_overview(
    query: AnalyticsQuery,
) -> Result<AnalyticsOverviewResponse, ServerFnError> {
    use backend::application::queries::analytics_service::AnalyticsQueryService;
    use backend::infrastructure::db::Database;

    let pool = expect_context::<Database>();
    let service = AnalyticsQueryService::new(pool.get_connection().clone());
    service
        .overview(query)
        .await
        .map_err(|e| ServerFnError::new(format!("查询统计概览失败: {}", e)))
}

#[server(
    name = FetchAdminAnalyticsTrends,
    prefix = "/api/admin/analytics",
    endpoint = "/trends",
)]
pub async fn fetch_admin_analytics_trends(
    query: AnalyticsQuery,
) -> Result<AnalyticsTrendResponse, ServerFnError> {
    use backend::application::queries::analytics_service::AnalyticsQueryService;
    use backend::infrastructure::db::Database;

    let pool = expect_context::<Database>();
    let service = AnalyticsQueryService::new(pool.get_connection().clone());
    service
        .trends(query)
        .await
        .map_err(|e| ServerFnError::new(format!("查询趋势失败: {}", e)))
}

#[server(
    name = FetchAdminAnalyticsBreakdown,
    prefix = "/api/admin/analytics",
    endpoint = "/breakdown",
)]
pub async fn fetch_admin_analytics_breakdown(
    query: AnalyticsQuery,
) -> Result<AnalyticsBreakdownResponse, ServerFnError> {
    use backend::application::queries::analytics_service::AnalyticsQueryService;
    use backend::infrastructure::db::Database;

    let pool = expect_context::<Database>();
    let service = AnalyticsQueryService::new(pool.get_connection().clone());
    service
        .breakdown(query)
        .await
        .map_err(|e| ServerFnError::new(format!("查询分布失败: {}", e)))
}
