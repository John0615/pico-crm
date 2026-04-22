use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct AfterSalesCase {
    pub uuid: String,
    pub order_uuid: String,
    pub operator_uuid: Option<String>,
    #[serde(default)]
    pub operator_name: Option<String>,
    pub case_type: String,
    pub description: String,
    pub status: String,
    pub refund_amount_cents: Option<i64>,
    pub refund_reason: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct CreateAfterSalesCaseRequest {
    pub case_type: String,
    pub description: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct UpdateAfterSalesRefundRequest {
    pub refund_amount_cents: Option<i64>,
    pub refund_reason: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct AfterSalesCaseRecord {
    pub uuid: String,
    pub case_uuid: String,
    pub operator_uuid: Option<String>,
    #[serde(default)]
    pub operator_name: Option<String>,
    pub content: String,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct CreateAfterSalesCaseRecordRequest {
    pub content: String,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct AfterSalesRework {
    pub uuid: String,
    pub case_uuid: String,
    pub assigned_user_uuid: String,
    #[serde(default)]
    pub assigned_user_name: Option<String>,
    pub scheduled_start_at: String,
    pub scheduled_end_at: String,
    pub note: Option<String>,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct CreateAfterSalesReworkRequest {
    pub assigned_user_uuid: String,
    pub scheduled_start_at: String,
    pub scheduled_end_at: String,
    pub note: Option<String>,
}
