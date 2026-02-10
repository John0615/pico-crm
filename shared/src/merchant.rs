use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ProvisionMerchantRequest {
    pub name: String,
    pub short_name: Option<String>,
    pub contact_name: String,
    pub contact_phone: String,
    pub merchant_type: Option<String>,
    pub plan_type: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ProvisionMerchantResponse {
    pub merchant_uuid: String,
    pub schema_name: String,
    pub status: String,
}
