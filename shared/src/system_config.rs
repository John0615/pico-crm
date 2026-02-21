use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SystemConfigCategoryDto {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub sort_order: i32,
    pub is_active: bool,
    pub items: Vec<SystemConfigItemDto>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SystemConfigItemDto {
    pub key: String,
    pub category_code: String,
    pub label: String,
    pub description: Option<String>,
    pub value_type: String,
    pub default_value: Value,
    pub value: Value,
    pub validation: Option<Value>,
    pub ui_schema: Option<Value>,
    pub is_required: bool,
    pub is_editable: bool,
    pub is_sensitive: bool,
    pub sort_order: i32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SystemConfigUpdateItem {
    pub key: String,
    pub value: Option<Value>,
    pub reset_to_default: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SystemConfigUpdateRequest {
    pub items: Vec<SystemConfigUpdateItem>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SystemConfigUpdateResponse {
    pub items: Vec<SystemConfigItemDto>,
}
