use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Contact {
    pub contact_uuid: String,
    pub user_name: String,
    pub company: String,
    pub position: String,
    pub phone_number: String,
    pub email: String,
    pub last_contact: String,
    pub value_level: i32,
    pub status: i32,
    pub inserted_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct SortOption {
    pub field: String,
    pub order: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct ContactFilters {
    pub user_name: Option<String>,
    pub status: Option<String>,
    pub email: Option<String>,
    pub phone_number: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct ContactQuery {
    pub page: u64,
    pub page_size: u64,
    pub sort: Option<Vec<SortOption>>,
    pub filters: Option<ContactFilters>,
}
