use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Order {
    pub uuid: String,
    pub request_id: Option<String>,
    pub contact_uuid: Option<String>,
    pub assigned_user_uuid: Option<String>,
    pub scheduled_start_at: Option<String>,
    pub scheduled_end_at: Option<String>,
    pub status: String,
    pub settlement_status: String,
    pub amount_cents: i64,
    pub notes: Option<String>,
    pub dispatch_note: Option<String>,
    pub settlement_note: Option<String>,
    pub inserted_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CreateOrderFromRequest {
    pub request_id: String,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UpdateOrderStatus {
    pub status: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UpdateOrderAssignment {
    pub assigned_user_uuid: Option<String>,
    pub scheduled_start_at: Option<String>,
    pub scheduled_end_at: Option<String>,
    pub dispatch_note: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UpdateOrderSettlement {
    pub settlement_status: String,
    pub settlement_note: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OrderQuery {
    pub page: u64,
    pub page_size: u64,
    pub status: Option<String>,
    pub contact_uuid: Option<String>,
    pub assigned_user_uuid: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}
