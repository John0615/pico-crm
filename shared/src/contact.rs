use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Contact {
    pub contact_uuid: String,
    pub user_name: String,
    pub phone_number: String,
    pub address: Option<String>,
    pub community: Option<String>,
    pub building: Option<String>,
    pub house_area_sqm: Option<i32>,
    pub service_need: Option<String>,
    pub tags: Vec<String>,
    pub last_service_at: Option<String>,
    pub follow_up_status: Option<String>,
    pub inserted_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct UpdateContact {
    pub contact_uuid: String,
    pub user_name: String,
    pub phone_number: String,
    pub address: Option<String>,
    pub community: Option<String>,
    pub building: Option<String>,
    pub house_area_sqm: Option<i32>,
    pub service_need: Option<String>,
    pub tags: Vec<String>,
    pub last_service_at: Option<String>,
    pub follow_up_status: Option<String>,
}

// 允许的排序字段枚举
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum SortField {
    Name,
}

// 排序方向枚举
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    Asc,
    Desc,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SortOption {
    pub field: SortField,
    pub order: SortOrder,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct ContactFilters {
    pub user_name: Option<String>,
    pub phone_number: Option<String>,
    pub address_keyword: Option<String>,
    pub tag: Option<String>,
    pub follow_up_status: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct ContactQuery {
    pub page: u64,
    pub page_size: u64,
    pub sort: Option<Vec<SortOption>>,
    pub filters: Option<ContactFilters>,
}
