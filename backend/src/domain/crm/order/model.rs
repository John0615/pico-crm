use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Order {
    pub uuid: String,
    pub request_id: Option<String>,
    pub customer_uuid: Option<String>,
    pub scheduled_start_at: Option<DateTime<Utc>>,
    pub scheduled_end_at: Option<DateTime<Utc>>,
    pub status: OrderStatus,
    pub cancellation_reason: Option<String>,
    pub completed_at: Option<DateTime<Utc>>,
    pub settlement_status: SettlementStatus,
    pub amount_cents: i64,
    pub paid_amount_cents: Option<i64>,
    pub payment_method: Option<String>,
    pub paid_at: Option<DateTime<Utc>>,
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

#[derive(Debug, Clone)]
pub struct OrderDetailsUpdate {
    pub customer_uuid: String,
    pub amount_cents: i64,
    pub notes: Option<String>,
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

    pub fn can_transition(current: OrderStatus, next: OrderStatus) -> bool {
        current == next
            || matches!(
                (current, next),
                (OrderStatus::Pending, OrderStatus::Confirmed)
                    | (OrderStatus::Pending, OrderStatus::Dispatching)
                    | (OrderStatus::Pending, OrderStatus::InService)
                    | (OrderStatus::Confirmed, OrderStatus::Dispatching)
                    | (OrderStatus::Confirmed, OrderStatus::InService)
                    | (OrderStatus::Dispatching, OrderStatus::Confirmed)
                    | (OrderStatus::Dispatching, OrderStatus::InService)
                    | (OrderStatus::InService, OrderStatus::Completed)
            )
    }

    pub fn validate_transition(current: OrderStatus, next: OrderStatus) -> Result<(), String> {
        if Self::can_transition(current, next) {
            Ok(())
        } else {
            Err(format!(
                "invalid order status transition: {} -> {}",
                current.as_str(),
                next.as_str()
            ))
        }
    }

    pub fn next_after_schedule_assignment(current: OrderStatus) -> OrderStatus {
        match current {
            OrderStatus::Pending | OrderStatus::Confirmed => OrderStatus::Dispatching,
            other => other,
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
    pub fn new_from_request(
        request_id: String,
        customer_uuid: String,
        scheduled_start_at: Option<DateTime<Utc>>,
        scheduled_end_at: Option<DateTime<Utc>>,
        notes: Option<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            uuid: Uuid::new_v4().to_string(),
            request_id: Some(request_id),
            customer_uuid: Some(customer_uuid),
            scheduled_start_at,
            scheduled_end_at,
            status: OrderStatus::Pending,
            cancellation_reason: None,
            completed_at: None,
            settlement_status: SettlementStatus::Unsettled,
            amount_cents: 0,
            paid_amount_cents: None,
            payment_method: None,
            paid_at: None,
            notes,
            dispatch_note: None,
            settlement_note: None,
            inserted_at: now,
            updated_at: now,
        }
    }

    pub fn verify(&self) -> Result<(), String> {
        if self
            .customer_uuid
            .as_ref()
            .map(|v| v.trim().is_empty())
            .unwrap_or(true)
        {
            return Err("Customer is required".to_string());
        }
        if self.amount_cents < 0 {
            return Err("Amount cents must be non-negative".to_string());
        }
        if let (Some(start), Some(end)) = (self.scheduled_start_at, self.scheduled_end_at) {
            if end <= start {
                return Err("Scheduled end must be after start".to_string());
            }
        }
        Ok(())
    }

    pub fn update_status(&mut self, status: OrderStatus) {
        self.status = status;
        if status != OrderStatus::Completed {
            self.completed_at = None;
        }
        self.updated_at = Utc::now();
    }

    pub fn update_assignment(&mut self, update: OrderAssignmentUpdate) -> Result<(), String> {
        if let (Some(start), Some(end)) = (update.scheduled_start_at, update.scheduled_end_at) {
            if end <= start {
                return Err("Scheduled end must be after start".to_string());
            }
        }
        self.scheduled_start_at = update.scheduled_start_at;
        self.scheduled_end_at = update.scheduled_end_at;
        self.dispatch_note = update.dispatch_note;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn update_details(&mut self, update: OrderDetailsUpdate) -> Result<(), String> {
        if self.status == OrderStatus::Completed || self.status == OrderStatus::Cancelled {
            return Err("Completed or cancelled orders cannot update core fields".to_string());
        }
        if update.customer_uuid.trim().is_empty() {
            return Err("Customer is required".to_string());
        }
        if update.amount_cents < 0 {
            return Err("Amount cents must be non-negative".to_string());
        }

        self.customer_uuid = Some(update.customer_uuid);
        self.amount_cents = update.amount_cents;
        self.notes = update.notes;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn cancel(&mut self, reason: String) -> Result<(), String> {
        if reason.trim().is_empty() {
            return Err("Cancellation reason is required".to_string());
        }
        if self.status == OrderStatus::Completed {
            return Err("Completed orders cannot be cancelled".to_string());
        }
        self.status = OrderStatus::Cancelled;
        self.cancellation_reason = Some(reason.trim().to_string());
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn update_settlement(
        &mut self,
        status: SettlementStatus,
        note: Option<String>,
        paid_amount_cents: Option<i64>,
        payment_method: Option<String>,
        paid_at: Option<DateTime<Utc>>,
    ) -> Result<(), String> {
        if let Some(amount) = paid_amount_cents {
            if amount < 0 {
                return Err("paid amount cents must be non-negative".to_string());
            }
        }
        self.settlement_status = status;
        self.settlement_note = note;
        self.paid_amount_cents = paid_amount_cents;
        self.payment_method = payment_method.and_then(|value| {
            let trimmed = value.trim().to_string();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        });
        self.paid_at = paid_at;
        self.updated_at = Utc::now();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_order() -> Order {
        Order::new_from_request(
            "request-1".to_string(),
            "customer-1".to_string(),
            None,
            None,
            Some("new".to_string()),
        )
    }

    #[test]
    fn order_status_transition_rules_allow_supported_paths() {
        assert!(
            OrderStatus::validate_transition(OrderStatus::Pending, OrderStatus::Confirmed).is_ok()
        );
        assert!(
            OrderStatus::validate_transition(OrderStatus::Confirmed, OrderStatus::Dispatching)
                .is_ok()
        );
        assert!(
            OrderStatus::validate_transition(OrderStatus::Dispatching, OrderStatus::InService)
                .is_ok()
        );
        assert!(
            OrderStatus::validate_transition(OrderStatus::Pending, OrderStatus::InService).is_ok()
        );
        assert!(
            OrderStatus::validate_transition(OrderStatus::InService, OrderStatus::Completed)
                .is_ok()
        );
    }

    #[test]
    fn order_status_transition_rules_reject_invalid_paths() {
        assert!(
            OrderStatus::validate_transition(OrderStatus::Pending, OrderStatus::Completed).is_err()
        );
        assert!(
            OrderStatus::validate_transition(OrderStatus::Completed, OrderStatus::Confirmed)
                .is_err()
        );
        assert!(
            OrderStatus::validate_transition(OrderStatus::Cancelled, OrderStatus::Pending).is_err()
        );
    }

    #[test]
    fn completed_orders_cannot_update_core_fields() {
        let mut order = sample_order();
        order.status = OrderStatus::Completed;

        let err = order
            .update_details(OrderDetailsUpdate {
                customer_uuid: "customer-2".to_string(),
                amount_cents: 200,
                notes: None,
            })
            .expect_err("completed orders should reject detail updates");
        assert!(err.contains("cannot update core fields"));
    }

    #[test]
    fn cancel_requires_reason() {
        let mut order = sample_order();
        assert!(order.cancel(" ".to_string()).is_err());
    }

    #[test]
    fn settlement_update_persists_payment_fields() {
        let mut order = sample_order();
        let paid_at = Utc::now();

        order
            .update_settlement(
                SettlementStatus::Settled,
                Some("微信收款".to_string()),
                Some(29900),
                Some("wechat".to_string()),
                Some(paid_at),
            )
            .expect("settlement update should succeed");

        assert_eq!(order.settlement_status, SettlementStatus::Settled);
        assert_eq!(order.paid_amount_cents, Some(29900));
        assert_eq!(order.payment_method.as_deref(), Some("wechat"));
        assert_eq!(order.paid_at, Some(paid_at));
    }

    #[test]
    fn settlement_update_rejects_negative_paid_amount() {
        let mut order = sample_order();
        let err = order
            .update_settlement(SettlementStatus::Settled, None, Some(-1), None, None)
            .expect_err("negative paid amount should be rejected");

        assert!(err.contains("paid amount"));
    }
}
