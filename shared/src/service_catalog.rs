use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct ServiceCatalog {
    pub uuid: String,
    pub name: String,
    pub description: Option<String>,
    pub base_price_cents: i64,
    pub default_duration_minutes: Option<i32>,
    pub is_active: bool,
    pub sort_order: i32,
    pub inserted_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct CreateServiceCatalogRequest {
    pub name: String,
    pub description: Option<String>,
    pub base_price_cents: i64,
    pub default_duration_minutes: Option<i32>,
    pub is_active: bool,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct UpdateServiceCatalogRequest {
    pub name: String,
    pub description: Option<String>,
    pub base_price_cents: i64,
    pub default_duration_minutes: Option<i32>,
    pub is_active: bool,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct ServiceCatalogQuery {
    pub active_only: Option<bool>,
}
