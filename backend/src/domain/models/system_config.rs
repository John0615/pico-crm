use chrono::{DateTime, Utc};
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct SystemConfigCategory {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub sort_order: i32,
    pub is_active: bool,
    pub items: Vec<SystemConfigItem>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct SystemConfigItem {
    pub uuid: String,
    pub category_code: String,
    pub key: String,
    pub label: String,
    pub description: Option<String>,
    pub value_type: String,
    pub default_value: Value,
    pub value: Option<Value>,
    pub validation: Option<Value>,
    pub ui_schema: Option<Value>,
    pub is_required: bool,
    pub is_editable: bool,
    pub is_sensitive: bool,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct SystemConfigItemUpdate {
    pub key: String,
    pub value: Value,
}

#[derive(Debug, Clone)]
pub struct SystemConfigItemUpdateRequest {
    pub key: String,
    pub value: Option<Value>,
    pub reset_to_default: bool,
}
