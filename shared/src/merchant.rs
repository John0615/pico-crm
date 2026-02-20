use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ProvisionMerchantRequest {
    pub name: String,
    pub short_name: Option<String>,
    pub contact_name: String,
    pub contact_phone: String,
    pub merchant_type: Option<String>,
    pub plan_type: Option<String>,
    pub owner_user_name: Option<String>,
    pub owner_password: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ProvisionMerchantResponse {
    pub merchant_uuid: String,
    pub schema_name: String,
    pub status: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MerchantListQuery {
    pub page: u64,
    pub page_size: u64,
    pub name: Option<String>,
    pub status: Option<String>,
    pub plan_type: Option<String>,
    pub contact_phone: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct MerchantSummary {
    pub uuid: String,
    pub name: String,
    pub short_name: Option<String>,
    pub schema_name: String,
    pub contact_name: String,
    pub contact_phone: String,
    pub merchant_type: Option<String>,
    pub plan_type: Option<String>,
    pub status: String,
    pub trial_end_at: Option<String>,
    pub expired_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MerchantPagedResult<T> {
    pub items: Vec<T>,
    pub total: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct UpdateMerchantRequest {
    pub name: Option<String>,
    pub short_name: Option<String>,
    pub contact_name: Option<String>,
    pub contact_phone: Option<String>,
    pub merchant_type: Option<String>,
    pub status: Option<String>,
    pub plan_type: Option<String>,
    pub trial_end_at: Option<String>,
    pub expired_at: Option<String>,
}
