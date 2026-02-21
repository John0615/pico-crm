use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Order {
    pub uuid: String,
    pub request_id: Option<String>,
    pub contact_uuid: Option<String>,
    pub assigned_user_uuid: Option<String>,
    pub scheduled_start_at: Option<DateTime<Utc>>,
    pub scheduled_end_at: Option<DateTime<Utc>>,
    pub status: OrderStatus,
    pub settlement_status: SettlementStatus,
    pub amount_cents: i64,
    pub notes: Option<String>,
    pub dispatch_note: Option<String>,
    pub settlement_note: Option<String>,
    pub inserted_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct OrderAssignmentUpdate {
    pub assigned_user_uuid: Option<String>,
    pub scheduled_start_at: Option<DateTime<Utc>>,
    pub scheduled_end_at: Option<DateTime<Utc>>,
    pub dispatch_note: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderStatus {
    Pending,
    Confirmed,
    Dispatching,
    InService,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettlementStatus {
    Unsettled,
    Settled,
}

impl OrderStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            OrderStatus::Pending => "pending",
            OrderStatus::Confirmed => "confirmed",
            OrderStatus::Dispatching => "dispatching",
            OrderStatus::InService => "in_service",
            OrderStatus::Completed => "completed",
            OrderStatus::Cancelled => "cancelled",
        }
    }

    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "pending" => Ok(OrderStatus::Pending),
            "confirmed" => Ok(OrderStatus::Confirmed),
            "dispatching" => Ok(OrderStatus::Dispatching),
            "in_service" => Ok(OrderStatus::InService),
            "completed" => Ok(OrderStatus::Completed),
            "cancelled" => Ok(OrderStatus::Cancelled),
            _ => Err(format!("Invalid order status: {}", value)),
        }
    }
}

impl SettlementStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            SettlementStatus::Unsettled => "unsettled",
            SettlementStatus::Settled => "settled",
        }
    }

    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "unsettled" => Ok(SettlementStatus::Unsettled),
            "settled" => Ok(SettlementStatus::Settled),
            _ => Err(format!("Invalid settlement status: {}", value)),
        }
    }
}

impl Order {
    pub fn new_from_request(request_id: String, contact_uuid: String, notes: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            uuid: Uuid::new_v4().to_string(),
            request_id: Some(request_id),
            contact_uuid: Some(contact_uuid),
            assigned_user_uuid: None,
            scheduled_start_at: None,
            scheduled_end_at: None,
            status: OrderStatus::Pending,
            settlement_status: SettlementStatus::Unsettled,
            amount_cents: 0,
            notes,
            dispatch_note: None,
            settlement_note: None,
            inserted_at: now,
            updated_at: now,
        }
    }

    pub fn verify(&self) -> Result<(), String> {
        if self.request_id.as_ref().map(|v| v.trim().is_empty()).unwrap_or(true) {
            return Err("Request id is required".to_string());
        }
        if self.contact_uuid.as_ref().map(|v| v.trim().is_empty()).unwrap_or(true) {
            return Err("Contact is required".to_string());
        }
        if let (Some(start), Some(end)) = (self.scheduled_start_at, self.scheduled_end_at) {
            if end < start {
                return Err("Scheduled end must be after start".to_string());
            }
        }
        Ok(())
    }

    pub fn update_status(&mut self, status: OrderStatus) {
        self.status = status;
        self.updated_at = Utc::now();
    }

    pub fn update_assignment(&mut self, update: OrderAssignmentUpdate) -> Result<(), String> {
        if let (Some(start), Some(end)) = (update.scheduled_start_at, update.scheduled_end_at) {
            if end < start {
                return Err("Scheduled end must be after start".to_string());
            }
        }
        self.assigned_user_uuid = update.assigned_user_uuid;
        self.scheduled_start_at = update.scheduled_start_at;
        self.scheduled_end_at = update.scheduled_end_at;
        self.dispatch_note = update.dispatch_note;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn update_settlement(&mut self, status: SettlementStatus, note: Option<String>) {
        self.settlement_status = status;
        self.settlement_note = note;
        self.updated_at = Utc::now();
    }
}
