use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Schedule {
    pub order_uuid: String,
    pub order_status: String,
    pub schedule_status: String,
    pub contact_uuid: Option<String>,
    pub assigned_user_uuid: Option<String>,
    pub scheduled_start_at: Option<String>,
    pub scheduled_end_at: Option<String>,
    pub dispatch_note: Option<String>,
    pub notes: Option<String>,
    pub inserted_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ScheduleQuery {
    pub page: u64,
    pub page_size: u64,
    pub status: Option<String>,
    pub assigned_user_uuid: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CreateScheduleAssignment {
    pub assigned_user_uuid: String,
    pub scheduled_start_at: String,
    pub scheduled_end_at: String,
    pub dispatch_note: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UpdateScheduleAssignment {
    pub assigned_user_uuid: Option<String>,
    pub scheduled_start_at: Option<String>,
    pub scheduled_end_at: Option<String>,
    pub dispatch_note: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UpdateScheduleStatus {
    pub status: String,
}
