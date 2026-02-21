use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct AnalyticsQuery {
    pub preset: Option<String>,
    pub start: Option<String>,
    pub end: Option<String>,
    pub timezone: Option<String>,
    pub granularity: Option<String>,
    pub dimension: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct AnalyticsMeta {
    pub start: String,
    pub end: String,
    pub timezone: String,
    pub granularity: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct AnalyticsOverview {
    pub total_merchants: u64,
    pub active_merchants: u64,
    pub total_users: u64,
    pub active_users: u64,
    pub new_merchants: u64,
    pub new_users: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct AnalyticsOverviewResponse {
    pub meta: AnalyticsMeta,
    pub overview: AnalyticsOverview,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AnalyticsTrendPoint {
    pub bucket: String,
    pub new_merchants: u64,
    pub new_users: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct AnalyticsTrendResponse {
    pub meta: AnalyticsMeta,
    pub series: Vec<AnalyticsTrendPoint>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AnalyticsBreakdownItem {
    pub label: String,
    pub count: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct AnalyticsBreakdownResponse {
    pub meta: AnalyticsMeta,
    pub dimension: String,
    pub items: Vec<AnalyticsBreakdownItem>,
}
