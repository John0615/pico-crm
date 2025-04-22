use serde::{Deserialize, Serialize};

// 定义排序字段
#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct SortField {
    pub user_name: bool,
    pub status: bool,
    pub last_contact: bool,
}

// 定义排序过滤条件
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SearchConditions {
    pub search_content: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ContactsParams {
    pub page: u32,
    pub page_size: u32,
    pub sort_fields: SortField,
    pub search_conditions: SearchConditions,
}

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
